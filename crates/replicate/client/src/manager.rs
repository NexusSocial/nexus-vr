//! Messages for the manager API.

use std::fmt::Debug;

use async_trait::async_trait;
use eyre::{bail, ensure, eyre, Context, OptionExt};
use eyre::{ContextCompat, Result};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use url::Url;

use crate::connect_to_url;
use crate::Ascii;

type Framed = replicate_common::Framed<wtransport::stream::BiStream, Cb, Sb>;

/// Manages instances on the instance server. Under the hood, this is all done
/// as reliable messages on top of a WebTransport connection.
///
/// Note: Clients must have permission to manage the server in order to connect.
/// This is initially going to be implemented as an allowlist in the server of
/// user IDs.
#[derive(Debug)]
pub struct Manager {
	pool: bb8::Pool<StreamPoolManager>,
	url: Url,
}

#[derive(Debug)]
struct StreamPoolManager {
	conn: wtransport::Connection,
}

impl StreamPoolManager {
	fn new(conn: wtransport::Connection) -> Self {
		Self { conn }
	}
}

// bb8 returns connections to the pool even if the drop is due to a panic.
// To avoid that, we drop the inner connection if the thread is panicking.
struct DropConnectionOnPanic<'a> {
	pooled_connection: bb8::PooledConnection<'a, StreamPoolManager>,
}

impl<'a> Drop for DropConnectionOnPanic<'a> {
	fn drop(&mut self) {
		if std::thread::panicking() {
			(*self.pooled_connection).take();
		}
	}
}

#[async_trait]
impl bb8::ManageConnection for StreamPoolManager {
	/// The connection type this manager deals with.
	type Connection = Option<Framed>;
	/// The error type returned by `Connection`s.
	type Error = eyre::Report;
	/// Attempts to create a new connection.
	async fn connect(&self) -> Result<Self::Connection, Self::Error> {
		let bi = wtransport::stream::BiStream::join(
			self.conn
				.open_bi()
				.await
				.wrap_err("could not initiate bi stream")?
				.await
				.wrap_err("could not finish opening bi stream")?,
		);

		let framed = Framed::new(bi);
		Ok(Some(framed))
	}
	/// Determines if the connection is still connected to the database.
	async fn is_valid(&self, framed: &mut Self::Connection) -> Result<(), Self::Error> {
		let framed = framed
			.as_mut()
			.wrap_err("connection was dropped due to panic")?;
		framed
			.send(Sb::HandshakeRequest)
			.await
			.wrap_err("failed to send handshake request")?;
		let Some(msg) = framed.next().await else {
			bail!("Server disconnected before completing handshake");
		};
		let msg = msg.wrap_err("error while receiving handshake response")?;
		ensure!(
			msg == Cb::HandshakeResponse,
			"invalid message during handshake"
		);
		Ok(())
	}
	/// Synchronously determine if the connection is no longer usable, if possible.
	fn has_broken(&self, framed: &mut Self::Connection) -> bool {
		framed.is_none()
	}
}

impl Manager {
	/// # Arguments
	/// - `url`: Url of the manager api. For example, `https://foobar.com/my/manager`
	///   or `192.168.1.1:1337/uwu/some_manager`.
	/// - `bearer_token`: optional, must be ascii otherwise we will panic.
	pub async fn connect(url: Url, bearer_token: Option<&str>) -> Result<Self> {
		let bearer_token = bearer_token.map(|s| {
			// Technically, bearer tokens only permit a *subset* of ascii. But I
			// didn't care enough to be that precise.
			Ascii::try_from(s).expect("to be in-spec, bearer tokens must be ascii")
		});
		let conn = connect_to_url(&url, bearer_token)
			.await
			.wrap_err("failed to connect to server")?;

		let manager = StreamPoolManager::new(conn);
		let pool = bb8::Pool::builder().build(manager).await.unwrap();

		Ok(Self { pool, url })
	}

	pub async fn instance_create(&self) -> Result<InstanceId> {
		let response = self.request(Sb::InstanceCreateRequest).await?;
		let Cb::InstanceCreateResponse { id } = response else {
			bail!("unexpected response: {response:?}");
		};
		Ok(id)
	}

	pub async fn instance_url(&self, id: InstanceId) -> Result<Url> {
		let response = self.request(Sb::InstanceUrlRequest { id }).await?;
		let Cb::InstanceUrlResponse { url } = response else {
			bail!("unexpected response: {response:?}");
		};
		Ok(url)
	}

	async fn get_framed(&self) -> Result<DropConnectionOnPanic<'_>> {
		let pooled_connection = self.pool.get().await.map_err(|e| match e {
			bb8::RunError::User(eyre) => {
				eyre.wrap_err("get from connection pool failed")
			}
			bb8::RunError::TimedOut => eyre!("connection pool fetch timed out"),
		})?;
		Ok(DropConnectionOnPanic { pooled_connection })
	}

	async fn request(&self, request: Sb) -> Result<Cb> {
		let mut wrapper = self.get_framed().await?;
		let framed = wrapper
			.pooled_connection
			.as_mut()
			.expect("only emptied in Drop impl");
		framed
			.send(request)
			.await
			.wrap_err("error while sending request")?;
		let response = framed
			.next()
			.await
			.ok_or_eyre("expected a response from the server")?
			.wrap_err("error while receiving response")?;
		Ok(response)
	}

	/// The url of this Manager.
	pub fn url(&self) -> &Url {
		&self.url
	}
}
