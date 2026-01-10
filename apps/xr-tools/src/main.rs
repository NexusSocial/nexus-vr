mod openxr;

use clap::Parser;
use color_eyre::Result;
use tracing::info;

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
		.with_writer(std::io::stderr)
		.init();
	let args = Args::parse();
	info!("started xr-tools");

	args.run()
}
