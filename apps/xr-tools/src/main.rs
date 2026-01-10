mod openxr;

use clap::Parser;
use color_eyre::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[clap(about)]
enum Args {
	Openxr(crate::openxr::Args),
}

impl Args {
	fn run(self) -> Result<()> {
		match self {
			Args::Openxr(a) => a.run(),
		}
	}
}

fn main() -> Result<()> {
	color_eyre::install()?;
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.with_writer(std::io::stderr)
		.init();
	let args = Args::parse();
	info!("started xr-tools");

	args.run()
}
