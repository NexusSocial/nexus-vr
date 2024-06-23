//! WebTransport server, i.e. "chad" transport.

use color_eyre::{eyre::Context, Result};
use eyre::{bail, ensure};
use futures::{sink::SinkExt, stream::StreamExt};
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use tracing::{info, instrument};
use url::Url;
use wtransport::endpoint::SessionRequest;

use crate::{ServerCtx, ServerCtxInner};

type Framed = replicate_common::Framed<wtransport::stream::BiStream, Sb, Cb>;

pub const URL_PREFIX: &str = "manager";

/// Entry point for the manager API
#[instrument(skip_all, name = "manager")]
pub async fn handle_connection(
	svr_ctx: ServerCtx,
	session_request: SessionRequest,
) -> Result<()> {
	let connection = session_request.accept().await?;
	info!("accepted wt connection to manager");
	let bi = wtransport::stream::BiStream::join(
		connection
			.accept_bi()
			.await
			.wrap_err("expected client to open bi stream")?,
	);
	let mut framed = Framed::new(bi);

	// Do handshake before anything else
	{
		let Some(msg) = framed.next().await else {
			bail!("Client disconnected before completing handshake");
		};
		let msg = msg.wrap_err("error while receiving handshake message")?;
		ensure!(
			msg == Sb::HandshakeRequest,
			"invalid message during handshake"
		);
		framed
			.send(Cb::HandshakeResponse)
			.await
			.wrap_err("failed to send handshake response")?;
	}

	while let Some(request) = framed.next().await {
		let request: Sb = request.wrap_err("error while receiving message")?;
		match request {
			Sb::InstanceCreateRequest => {
				// TODO: Actually manipulate the instance manager.
				let response = Cb::InstanceCreateResponse {
					id: InstanceId::random(),
				};
				framed.send(response).await?;
			}
			Sb::InstanceUrlRequest { id } => {
				let url = {
					let svr_ctx_l = svr_ctx.0.read().expect("poisoned");
					let domain_name =
						svr_ctx_l.san.first().expect("should have domain name");
					let port = svr_ctx_l.port;
					let hash = &svr_ctx_l.cert.base64_hash;
					// TODO: Actually manipulate the instance manager.
					Url::parse(&format!(
						"https://{domain_name}:{port}/{}/{id}/#{hash}",
						crate::instance::URL_PREFIX,
					))
					.expect("invalid url")
				};
				let response = Cb::InstanceUrlResponse { url };
				framed.send(response).await?;
			}
			Sb::HandshakeRequest => {
				bail!("already did handshake, another handshake is unexpected")
			}
		}
	}

	info!("Client disconnected");
	Ok(())
}

pub fn manager_url(svr_ctx: &ServerCtxInner) -> String {
	let encoded_cert_hash = &svr_ctx.cert.base64_hash;
	let subject_alt_name = svr_ctx.san.first().expect("should have at least 1 SAN");
	let port = svr_ctx.port;
	format!(
		"https://{subject_alt_name}:{port}/{}/#{encoded_cert_hash}",
		URL_PREFIX
	)
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use tokio::time::Instant;

	// Sanity check.
	#[tokio::test]
	async fn test_that_interval_immediately_ticks() {
		let mut interval = tokio::time::interval(Duration::from_millis(500));
		let start = Instant::now();
		interval.tick().await;
		assert!(start.elapsed() < Duration::from_millis(5));
	}
}
