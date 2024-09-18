use std::net::{Ipv6Addr, SocketAddr};

use clap::Parser as _;
use color_eyre::eyre::Context as _;
use identity_server::{google_jwks_provider::JwksProvider, MigratedDbPool};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
struct Cli {
	#[clap(long, short, default_value = "0")]
	port: u16,
	#[clap(long, env, default_value = "identities.db")]
	db_path: PathBuf,
	/// The Google API OAuth2 Client ID.
	/// See https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid
	#[clap(long, env)]
	google_client_id: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let cli = Cli::parse();

	let db_pool = {
		let connect_opts = sqlx::sqlite::SqliteConnectOptions::new()
			.create_if_missing(true)
			.filename(&cli.db_path);
		let pool_opts = sqlx::sqlite::SqlitePoolOptions::new();
		let pool = pool_opts
			.connect_with(connect_opts.clone())
			.await
			.wrap_err_with(|| {
				format!(
					"failed to connect to database with path {}",
					connect_opts.get_filename().display()
				)
			})?;
		MigratedDbPool::new(pool)
			.await
			.wrap_err("failed to migrate db pool")?
	};
	let reqwest_client = reqwest::Client::new();

	let v1_cfg = identity_server::v1::RouterConfig {
		uuid_provider: Default::default(),
		db_pool,
	};
	let oauth_cfg = identity_server::oauth::OAuthConfig {
		google_client_id: cli.google_client_id,
		google_jwks_provider: JwksProvider::google(reqwest_client.clone()),
	};
	let router = identity_server::RouterConfig {
		v1: v1_cfg,
		oauth: oauth_cfg,
	}
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
