mod linux;
mod windows;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use crate::linux::linux;
use crate::windows::windows;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands
}

#[derive(Subcommand)]
enum Commands {
	All,
	Windows,
	Linux,
	MacIntel,
	MacArm,
	Android
}

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Commands::All => {
			linux();
			windows();
		}
		Commands::Windows => {
			windows();
		},
		Commands::Linux => {
			linux();
		}
		Commands::MacIntel => todo!(),
		Commands::MacArm => todo!(),
		Commands::Android => todo!(),
	}
}

