//! WebTransport server, i.e. "chad" transport.

use std::{num::Wrapping, sync::Arc, time::Duration};

use color_eyre::{eyre::Context, Result};
use tracing::{error, info, info_span, Instrument};
use wtransport::{endpoint::IncomingSession, Certificate, ServerConfig};

use crate::{instance::InstanceManager, Args};

type Server = wtransport::Endpoint<wtransport::endpoint::endpoint_side::Server>;

const CERT_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

pub async fn launch_webtransport_server(
	args: Args,
	_im: Arc<InstanceManager>,
) -> Result<()> {
	let mut cert = Certificate::self_signed(args.subject_alt_names.iter());
	let server = Server::server(
		ServerConfig::builder()
			.with_bind_default(args.port.unwrap_or(0))
			.with_certificate(cert.clone())
			.build(),
	)
	.wrap_err("failed to create wtransport server")?;

	info!("server ready");

	let mut id = Wrapping(0u64);
	let accept_fut = async {
		loop {
			let incoming = server.accept().await;
			id += 1;
			tokio::spawn(
				async move {
					if let Err(err) = handle_connection(incoming).await {
						error!("terminated with error: {err}");
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
			if let Err(err) = server
				.reload_config(
					ServerConfig::builder()
						.with_bind_default(args.port.unwrap_or(0))
						.with_certificate(cert)
						.build(),
					false,
				)
				.wrap_err("failed to reload server config")
			{
				return Err(err);
			}
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

	loop {
		let dgram = connection.receive_datagram().await?;
		info!("dgram: {dgram:?}");
	}
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
