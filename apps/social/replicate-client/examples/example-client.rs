use clap::Parser;
use color_eyre::Result;
use replicate_client::manager::Manager;
use replicate_common::did::{AuthenticationAttestation, Did, DidPrivateKey};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use url::Url;

/// Runs a nexus server.
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {
	#[clap(long)]
	url: Url,
	#[clap(long)]
	username: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let env_filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env_lossy();

	tracing_subscriber::fmt()
		.with_target(true)
		.with_level(true)
		.with_env_filter(env_filter)
		.init();

	let args = Args::parse();

	let did = Did(args.username);
	let did_private_key = DidPrivateKey;

	let auth_attest = AuthenticationAttestation::new(did, &did_private_key);

	let _manager = Manager::connect(args.url, auth_attest).await?;

	Ok(())
}
