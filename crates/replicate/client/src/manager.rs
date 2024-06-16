//! Messages for the manager API.

use std::fmt::Debug;

use eyre::{bail, ensure, eyre, Context};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use url::Url;

use crate::connect_to_url;
use crate::Ascii;

type Result<T> = eyre::Result<T>;
type Framed = replicate_common::Framed<wtransport::stream::BiStream, Cb, Sb>;

/// Manages instances on the instance server. Under the hood, this is all done
/// as reliable messages on top of a WebTransport connection.
///
/// Note: Clients must have permission to manage the server in order to connect.
/// This is initially going to be implemented as an allowlist in the server of
/// user IDs.
#[derive(Debug)]
pub struct Manager {
	_conn: wtransport::Connection,
	_url: Url,
	framed: Framed,
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
		let bi = wtransport::stream::BiStream::join(
			conn.open_bi()
				.await
				.wrap_err("could not initiate bi stream")?
				.await
				.wrap_err("could not finish opening bi stream")?,
		);

		let mut framed = Framed::new(bi);

		// Do handshake before anything else
		{
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
		}

		Ok(Self {
			_conn: conn,
			_url: url,
			framed,
		})
	}

	pub async fn instance_create(&mut self) -> Result<InstanceId> {
		self.framed
			.send(Sb::InstanceCreateRequest)
			.await
			.wrap_err("failed to write message")?;
		match self.framed.next().await {
			None => Err(eyre!("server disconnected")),
			Some(Err(err)) => {
				Err(eyre::Report::new(err).wrap_err("failed to receive message"))
			}
			Some(Ok(Cb::InstanceCreateResponse { id })) => Ok(id),
			Some(Ok(_)) => Err(eyre!("unexpected response")),
		}
	}

	pub async fn instance_url(&mut self, id: InstanceId) -> Result<Url> {
		self.framed
			.send(Sb::InstanceUrlRequest { id })
			.await
			.wrap_err("failed to write message")?;
		match self.framed.next().await {
			None => Err(eyre!("server disconnected")),
			Some(Err(err)) => {
				Err(eyre::Report::new(err).wrap_err("failed to receive message"))
			}
			Some(Ok(Cb::InstanceUrlResponse { url })) => Ok(url),
			Some(Ok(_)) => Err(eyre!("unexpected response")),
		}
	}
}
