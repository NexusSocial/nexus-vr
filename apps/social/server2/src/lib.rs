#![warn(clippy::option_unwrap_used)]

mod chad;
mod instance_manager;
mod trad;

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use futures_lite::FutureExt;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

/// Runs a nexus server.
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {
	/// Sets a custom port number
	#[clap(short, long)]
	port: Option<u16>,

	/// Sets a list of domain names for the certificate. The first one will be
	/// used as the domain name shown to users in URLs generated by the server.
	#[clap(short, long, default_value = "localhost")]
	subject_alt_names: Vec<String>,
}

pub async fn main(args: Args) -> Result<()> {
	color_eyre::install()?;
	let env_filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env_lossy();

	tracing_subscriber::fmt()
		.with_target(true)
		.with_level(true)
		.with_env_filter(env_filter)
		.init();

	let (im_handle, im_join) = crate::instance_manager::spawn();
	let im_join = async {
		Ok(im_join
			.await
			.wrap_err("instance_manager task failed to join")?
			.wrap_err("instance_manager task terminated with error")?)
	};
	let chad_fut = crate::chad::launch_webtransport_server(args.clone());
	let trad_fut = crate::trad::launch_http_server(im_handle, args.clone());

	chad_fut.race(trad_fut).race(im_join).await
}
