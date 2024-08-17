use std::net::{Ipv6Addr, SocketAddr};

use clap::Parser as _;
use color_eyre::eyre::Context as _;
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
struct Cli {
	#[clap(long, short, default_value = "0")]
	port: u16,
	#[clap(long, env, default_value = "identities.db")]
	db_path: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let cli = Cli::parse();
	// Validate cli args further
	if !cli.db_path.exists() {
		warn!(
			"no file exists at {}, creating a new database",
			cli.db_path.display()
		);
		tokio::fs::OpenOptions::new()
			.append(true)
			.create_new(true)
			.open(&cli.db_path)
			.await
			.wrap_err("failed to create empty file for new database")?;
	}

	let v1_cfg = identity_server::v1::RouterConfig {
		db_url: format!("sqlite:{}", cli.db_path.display()),
		..Default::default()
	};
	let router = identity_server::RouterConfig { v1: v1_cfg }
		.build()
		.await
		.wrap_err("failed to build router")?;

	let listener = tokio::net::TcpListener::bind(SocketAddr::new(
		Ipv6Addr::UNSPECIFIED.into(),
		cli.port,
	))
	.await
	.unwrap();
	info!("listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, router).await.map_err(|e| e.into())
}
