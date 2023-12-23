use clap::Parser;

/// This is my awesome program that does great things in headless mode or not
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
	/// Activate headless mode
	#[arg(long)]
	pub(crate) headless: bool,
	//#[arg(short, long)]
	//flatscreen: bool,
}
