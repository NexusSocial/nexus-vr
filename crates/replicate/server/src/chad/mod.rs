//! WebTransport server, i.e. "chad" transport.

use std::{
	num::Wrapping,
	sync::{Arc, RwLock},
	time::Duration,
};

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use color_eyre::{eyre::Context, Result};
use eyre::{bail, ensure};
use futures::{sink::SinkExt, stream::StreamExt};
use replicate_common::{
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use tracing::{error, info, info_span, Instrument};
use url::Url;
use wtransport::{endpoint::IncomingSession, ServerConfig};

use crate::{instance::InstanceManager, Args};

type Server = wtransport::Endpoint<wtransport::endpoint::endpoint_side::Server>;
type Framed = replicate_common::Framed<wtransport::stream::BiStream, Sb, Cb>;

const CERT_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

pub async fn launch_webtransport_server(
	args: Args,
	_im: Arc<InstanceManager>,
) -> Result<()> {
	let cert = Certificate::new(wtransport::Certificate::self_signed(
		args.subject_alt_names.iter(),
	));
	let server = Server::server(
		ServerConfig::builder()
			.with_bind_default(args.port.unwrap_or(0))
			.with_certificate(cert.cert.clone())
			.build(),
	)
	.wrap_err("failed to create wtransport server")?;

	let port = server
		.local_addr()
		.expect("could not determine port")
		.port();

	let svr_ctx = ServerCtx::new(ServerCtxInner {
		san: args.subject_alt_names,
		port,
		cert,
	});

	{
		let svr_ctx = svr_ctx.0.read().expect("lock poisoned");
		info!("server url:\n{}", server_url(&svr_ctx));
	}

	let mut id = Wrapping(0u64);
	let accept_fut = async {
		loop {
			let incoming = server.accept().await;
			let svr_ctx_clone = svr_ctx.clone();
			id += 1;
			tokio::spawn(
				async move {
					if let Err(err) = handle_connection(svr_ctx_clone, incoming).await {
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
			let mut svr_ctx_l = svr_ctx.0.write().expect("server context poisoned");
			svr_ctx_l.cert =
				wtransport::Certificate::self_signed(svr_ctx_l.san.iter()).into();

			#[allow(clippy::question_mark)] // false positive
			if let Err(err) = server
				.reload_config(
					ServerConfig::builder()
						.with_bind_default(args.port.unwrap_or(0))
						.with_certificate(svr_ctx_l.cert.cert.clone())
						.build(),
					false,
				)
				.wrap_err("failed to reload server config")
			{
				return Err(err);
			}
			info!("new server url:\n{}", server_url(&svr_ctx_l));
		}
	}
	.instrument(info_span!("cert refresh task"));
	tokio::select! {
		_ = accept_fut => unreachable!(),
		result = refresh_cert_fut => result?,
	}
}

async fn handle_connection(
	svr_ctx: ServerCtx,
	incoming: IncomingSession,
) -> Result<()> {
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
			Sb::InstanceUrlRequest { id } => {
				let url = {
					let svr_ctx_l = svr_ctx.0.read().expect("poisoned");
					let domain_name =
						svr_ctx_l.san.first().expect("should have domain name");
					let port = svr_ctx_l.port;
					let hash = &svr_ctx_l.cert.base64;
					// TODO: Actually manipulate the instance manager.
					Url::parse(&format!("https://{domain_name}:{port}/{id}/#{hash}"))
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

fn server_url(svr_ctx: &ServerCtxInner) -> String {
	let encoded_cert_hash = &svr_ctx.cert.base64;
	let subject_alt_name = svr_ctx.san.first().expect("should have at least 1 SAN");
	let port = svr_ctx.port;
	format!("https://{subject_alt_name}:{port}/#{encoded_cert_hash}")
}

/// Server state, concurrently shared across all connections
#[derive(Debug, Clone)]
struct ServerCtxInner {
	san: Vec<String>,
	port: u16,
	cert: Certificate,
}

#[derive(Debug, Clone)]
struct ServerCtx(Arc<RwLock<ServerCtxInner>>);

impl ServerCtx {
	fn new(ctx: ServerCtxInner) -> Self {
		Self(Arc::new(RwLock::new(ctx)))
	}
}

/// Connection state
#[derive(Debug)]
struct ConnectionCtx {}

/// Newtype on [`wtransport::Certificate`].
#[derive(Clone)]
struct Certificate {
	cert: wtransport::Certificate,
	base64: String,
}

impl Certificate {
	fn new(cert: wtransport::Certificate) -> Self {
		let cert_hash = cert.hashes().pop().expect("should be at least one hash");
		let base64 = BASE64_URL_SAFE_NO_PAD.encode(cert_hash.as_ref());
		Self { cert, base64 }
	}
}

impl From<wtransport::Certificate> for Certificate {
	fn from(value: wtransport::Certificate) -> Self {
		Self::new(value)
	}
}

impl std::fmt::Debug for Certificate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple(std::any::type_name::<Self>())
			.field(&self.cert.certificates())
			.finish()
	}
}

impl std::fmt::Display for Certificate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_list()
			.entries(
				self.cert
					.hashes()
					.into_iter()
					.map(|h| h.fmt(wtransport::tls::Sha256DigestFmt::DottedHex)),
			)
			.finish()
	}
}

impl AsRef<wtransport::Certificate> for Certificate {
	fn as_ref(&self) -> &wtransport::Certificate {
		&self.cert
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
