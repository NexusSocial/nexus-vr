use std::net::{Ipv6Addr, SocketAddr};

use clap::Parser as _;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
struct Cli {
	#[clap(default_value = "0")]
	port: u16,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let cli = Cli::parse();

	let listener = tokio::net::TcpListener::bind(SocketAddr::new(
		Ipv6Addr::UNSPECIFIED.into(),
		cli.port,
	))
	.await
	.unwrap();
	info!("listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, identity_server::router())
		.await
		.map_err(|e| e.into())
}
