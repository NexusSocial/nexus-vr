//! WebTransport server, i.e. "chad" transport.

use std::{num::Wrapping, sync::Arc, time::Duration};

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use color_eyre::{eyre::Context, Result};
use eyre::{bail, ensure};
use futures::{sink::SinkExt, stream::StreamExt};
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use tracing::{error, info, info_span, Instrument};
use wtransport::{endpoint::IncomingSession, Certificate, ServerConfig};

use crate::{instance::InstanceManager, Args};

type Server = wtransport::Endpoint<wtransport::endpoint::endpoint_side::Server>;
type Framed = replicate_common::Framed<wtransport::stream::BiStream, Sb, Cb>;

const CERT_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

pub async fn launch_webtransport_server(
	args: Args,
	_im: Arc<InstanceManager>,
) -> Result<()> {
	let mut cert = Certificate::self_signed(args.subject_alt_names.iter());
	let domain_name = args
		.subject_alt_names
		.first()
		.expect("should be at least one SAN");
	let server = Server::server(
		ServerConfig::builder()
			.with_bind_default(args.port.unwrap_or(0))
			.with_certificate(cert.clone())
			.build(),
	)
	.wrap_err("failed to create wtransport server")?;

	let port = server
		.local_addr()
		.expect("could not determine port")
		.port();

	info!("server url:\n{}", server_url(domain_name, port, &cert));

	let mut id = Wrapping(0u64);
	let accept_fut = async {
		loop {
			let incoming = server.accept().await;
			id += 1;
			tokio::spawn(
				async move {
					if let Err(err) = handle_connection(incoming).await {
						error!("terminated with error: {err:?}");
					} else {
						info!("disconnected");
					}
				}
				.instrument(info_span!("connection", id)),
			);
		}
	};

	let mut interval = tokio::time::interval(CERT_REFRESH_INTERVAL);
	let refresh_cert_fut = async {
		// tick it once to clear the initial tick
		interval.tick().await;
		loop {
			interval.tick().await;
			info!("refreshing certs");
			cert = Certificate::self_signed(args.subject_alt_names.iter());
			#[allow(clippy::question_mark)]
			if let Err(err) = server
				.reload_config(
					ServerConfig::builder()
						.with_bind_default(args.port.unwrap_or(0))
						.with_certificate(cert.clone())
						.build(),
					false,
				)
				.wrap_err("failed to reload server config")
			{
				return Err(err);
			}
			info!("new server url:\n{}", server_url(domain_name, port, &cert));
		}
	}
	.instrument(info_span!("cert refresh task"));
	tokio::select! {
		_ = accept_fut => unreachable!(),
		result = refresh_cert_fut => result?,
	}
}

async fn handle_connection(incoming: IncomingSession) -> Result<()> {
	info!("Waiting for session request...");
	let session_request = incoming.await?;
	info!(
		"New session: Authority: '{}', Path: '{}'",
		session_request.authority(),
		session_request.path()
	);
	let connection = session_request.accept().await?;
	info!("accepted wt connection");
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
			Sb::HandshakeRequest => {
				bail!("already did handshake, another handshake is unexpected")
			}
		}
	}

	info!("Client disconnected");
	Ok(())
}

fn server_url(subject_alt_name: &str, port: u16, cert: &Certificate) -> String {
	let cert_hash = cert.hashes().pop().expect("should be at least one hash");
	let encoded_cert_hash = BASE64_URL_SAFE_NO_PAD.encode(cert_hash.as_ref());
	format!("https://{subject_alt_name}:{port}/#{encoded_cert_hash}")
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
