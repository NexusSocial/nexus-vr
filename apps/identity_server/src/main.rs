use std::net::{Ipv6Addr, SocketAddr};

use clap::Parser as _;
use color_eyre::eyre::{ensure, Context as _};
use futures::StreamExt;
use identity_server::MigratedDbPool;
use rustls_acme::AcmeConfig;
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
struct Cli {
	#[clap(long, short, default_value = "0")]
	port: u16,
	#[clap(long, env, default_value = "identities.db")]
	db_path: PathBuf,
	#[clap(long, env)]
	tls_email: Option<String>,
	#[clap(long, env)]
	domains: Vec<String>,
	#[clap(long)]
	prod_tls: bool,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;
	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let cli = Cli::parse();

	ensure!(
		cli.domains.len() != 0,
		"must specify at least one server domain"
	);

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

	let v1_cfg = identity_server::v1::RouterConfig {
		uuid_provider: Default::default(),
		db_pool,
	};
	let router = identity_server::RouterConfig { v1: v1_cfg }
		.build()
		.await
		.wrap_err("failed to build router")?;

	let home_dir = std::env::var("HOME")
		.map(PathBuf::from)
		.unwrap_or(std::env::current_dir().wrap_err("no home dir")?);
	let cache_dir = std::env::var("XDG_CACHE_HOME")
		.map(PathBuf::from)
		.unwrap_or(home_dir.join(".cache"))
		.join("socialvr_identity_server");
	debug!("using cert cache dir {}", cache_dir.display());
	// .join(if cli.prod_tls { "prod" } else { "dev" });
	tokio::fs::create_dir_all(&cache_dir)
		.await
		.wrap_err("failed to create cache directory for certs")?;
	let acceptor = cli.tls_email.map(|email| {
		let mut state = AcmeConfig::new(cli.domains)
			.contact([format!("mailto:{email}")])
			// TODO: use tempdir for cache
			.cache_option(Some(rustls_acme::caches::DirCache::new(cache_dir)))
			.directory_lets_encrypt(cli.prod_tls) // TODO: Change to prod letsencrypt (true)
			.state();
		let acceptor = state.axum_acceptor(state.default_rustls_config());

		tokio::spawn(async move {
			loop {
				match state.next().await.unwrap() {
					Ok(ok) => tracing::info!("event: {:?}", ok),
					Err(err) => tracing::error!("error: {:?}", err),
				}
			}
		});

		acceptor
	});

	let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, cli.port));
	// info!("listening on {}", listener.local_addr().unwrap());
	info!("listening on port {}...", cli.port);
	let server = axum_server::bind(addr);
	if let Some(acceptor) = acceptor {
		server
			.acceptor(acceptor)
			.serve(router.into_make_service())
			.await
	} else {
		server.serve(router.into_make_service()).await
	}
	.wrap_err("server died")
}
