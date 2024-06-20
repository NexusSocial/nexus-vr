//! Messages for the manager API.

use std::fmt::Debug;

use eyre::Result;
use eyre::{bail, ensure, Context, OptionExt};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use tokio::sync::{mpsc, oneshot};
use url::Url;

use crate::connect_to_url;
use crate::Ascii;

/// The number of queued rpc calls allowed before we start erroring.
const RPC_CAPACITY: usize = 64;

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
	task: tokio::task::JoinHandle<Result<()>>,
	request_tx: mpsc::Sender<(Sb, oneshot::Sender<Cb>)>,
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

		let (request_tx, request_rx) = mpsc::channel(RPC_CAPACITY);
		let task = tokio::spawn(manager_task(framed, request_rx));

		Ok(Self {
			_conn: conn,
			_url: url,
			task,
			request_tx,
		})
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

	/// Panics if the connection is already dead
	async fn request(&self, request: Sb) -> Result<Cb> {
		let (response_tx, response_rx) = oneshot::channel();
		self.request_tx
			.send((request, response_tx))
			.await
			.wrap_err("failed to send to manager task")?;
		response_rx
			.await
			.wrap_err("failed to receive from manager task")
	}

	/// Destroys the manager and reaps any errors from its networking task
	pub async fn join(self) -> Result<()> {
		self.task
			.await
			.wrap_err("panic in manager task, file a bug report on github uwu")?
			.wrap_err("error in task")
	}
}

async fn manager_task(
	mut framed: Framed,
	mut request_rx: mpsc::Receiver<(Sb, oneshot::Sender<Cb>)>,
) -> Result<()> {
	while let Some((request, response_tx)) = request_rx.recv().await {
		framed
			.send(request)
			.await
			.wrap_err("error while sending request")?;
		let response = framed
			.next()
			.await
			.ok_or_eyre("expected a response from the server")?
			.wrap_err("error while receiving response")?;
		let _ = response_tx.send(response);
	}
	// We only return ok when the manager struct was dropped
	Ok(())
}
