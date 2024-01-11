//! HTTP server, i.e. "trad" transport.
//!
//! Major versions of the api are under v*, so that we can continue to support
//! old apis while the old servers migrate to the new api.
//!
//! For now, we are keeping all code on v0, since we don't want to commit to stability,
//! but we have broken it into a separate module to indicate that we should design with
//! the expectation that more versions will be added in the future.

mod v0;

use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing::get, Router};
use color_eyre::{eyre::Context, Result};
use tracing::info;

use crate::Args;

pub async fn launch_http_server(
	instance_manager: crate::instance_manager::Handle,
	args: Args,
) -> Result<()> {
	let port = args.port.unwrap_or(0);
	let sock_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port);

	let app = Router::new()
		.nest(
			"/api/v0",
			v0::router().with_state(v0::ApiState { instance_manager }),
		)
		.route("/", get(root));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind(sock_addr)
		.await
		.wrap_err("Failed to create tcp listener")?;
	let sock_addr = listener.local_addr()?;

	info!("starting http server at address {sock_addr}");
	axum::serve(listener, app)
		.await
		.wrap_err("err in http server")
}

async fn root() -> &'static str {
	"hello world"
}
