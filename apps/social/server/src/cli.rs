use clap::Parser;

/// This is my awesome program that does great things in headless mode or not
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	/// Activate headless mode
	#[arg(long)]
	pub headless: bool,
	//#[arg(short, long)]
	//flatscreen: bool,
	/// Print frame timings to console
	#[arg(long)]
	pub frame_timings: bool,
}
