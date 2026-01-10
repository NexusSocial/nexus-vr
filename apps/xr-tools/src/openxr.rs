use clap::Parser;
use color_eyre::{Result, eyre::Context, owo_colors::OwoColorize};
use tracing::debug;

#[derive(Debug, Parser)]
pub struct Args {
	#[clap(long)]
	static_loader: bool,
	#[clap(subcommand)]
	cmds: Subcommands,
}

#[derive(Debug, Parser)]
enum Subcommands {
	Info(InfoArgs),
}

#[derive(Debug, Parser)]
struct InfoArgs {}

impl InfoArgs {
	fn run(self, entry: ::openxr::Entry) -> Result<()> {
		println!("{}", "layers:".bold());
		for layer in entry
			.enumerate_layers()
			.wrap_err("failed to enumerate layers")?
		{
			println!("layer: {layer:?}");
		}

		let extensions = entry
			.enumerate_extensions()
			.wrap_err("failed to enumerate extensions")?;
		println!("{}", "extensions:".bold());
		for name in extensions.names() {
			let name = std::ffi::CStr::from_bytes_with_nul(name)
				.wrap_err("encountered name that was not a CStr")?;
			let name = std::str::from_utf8(name.to_bytes())
				.wrap_err("encountered name that was not UTF-8")?;
			println!("{name}");
		}

		Ok(())
	}
}

impl Args {
	pub fn run(self) -> Result<()> {
		let entry = if self.static_loader {
			::openxr::Entry::linked()
		} else {
			// SAFETY: YOLO
			unsafe { ::openxr::Entry::load() }
				.wrap_err("failed to dlopen openxr loader")?
		};
		debug!("got openxr loader");

		match self.cmds {
			Subcommands::Info(a) => a.run(entry),
		}
	}
}
