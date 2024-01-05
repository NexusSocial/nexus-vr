//! HTTP server, i.e. "trad" transport.

use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing::get, Router};
use color_eyre::{eyre::Context, Result};
use tracing::info;

use crate::Args;

pub async fn launch_http_server(args: Args) -> Result<()> {
	let port = args.port.unwrap_or(0);
	let sock_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port);
	let app = Router::new().route("/", get(root));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind(sock_addr).await.unwrap();
	let sock_addr = listener.local_addr()?;

	info!("starting http server at address {sock_addr}");
	axum::serve(listener, app)
		.await
		.wrap_err("err in http server")
}

async fn root() -> &'static str {
	"hello world"
}
