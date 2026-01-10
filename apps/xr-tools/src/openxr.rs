use clap::Parser;
use color_eyre::{Result, eyre::Context, owo_colors::OwoColorize};
use openxr::{
	ApplicationInfo, ExtensionSet, FormFactor, FrameStream, FrameWaiter, Graphics,
	Headless, Session, headless::SessionCreateInfo as HeadlessSessionCreateInfo,
};
use tracing::{debug, error, info};

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
		let mut layers = entry
			.enumerate_layers()
			.wrap_err("failed to enumerate layers")?;
		layers.sort_by_key(|l| l.layer_name.to_owned());
		println!("{}", "layers:".bold());
		for layer in layers {
			let name = &layer.layer_name;
			println!(
				"- {} v{}: spec version: {}, desc: {}",
				name, layer.layer_version, layer.spec_version, layer.description
			);

			let extensions =
				entry.enumerate_layer_extensions(name).wrap_err_with(|| {
					format!("failed to enumerate layer extensions for layer {name}")
				})?;
			let mut extension_names: Vec<_> = extensions
				.names()
				.into_iter()
				.map(String::from_utf8_lossy)
				.collect();
			extension_names.sort();
			for ext_name in extension_names {
				println!("  - {ext_name}");
			}
		}

		let supported_extensions = entry
			.enumerate_extensions()
			.wrap_err("failed to enumerate extensions")?;
		let mut names: Vec<_> = supported_extensions
			.names()
			.into_iter()
			.map(String::from_utf8_lossy)
			.collect();
		names.sort();
		println!("{}", "extensions:".bold());
		for name in names {
			println!("- {name}");
		}

		let mut requested_extensions = ExtensionSet::default();
		requested_extensions.mnd_headless = supported_extensions.mnd_headless;
		let mk_instance = move |ext| {
			entry
				.create_instance(
					&ApplicationInfo {
						application_name: "xr-tools",
						..Default::default()
					},
					&ext,
					&[],
				)
				.wrap_err("failed to create openxr instance")
		};

		let instance = match mk_instance(requested_extensions.clone()) {
			Ok(i) => i,
			Err(err) => {
				error!("failed to create instance with headless extension: {err}");
				info!("attempting again but this time with no extensions");
				requested_extensions = ExtensionSet::default();
				mk_instance(requested_extensions.clone())?
			}
		};
		debug!("created openxr instance");

		let runtime_properties = instance
			.properties()
			.wrap_err("failed to get runtime properties from instance")?;
		println!(
			"{} {}, {} {}",
			"runtime:".bold(),
			runtime_properties.runtime_name,
			"version:".bold(),
			runtime_properties.runtime_version
		);

		let hmd_system_id = instance
			.system(FormFactor::HEAD_MOUNTED_DISPLAY)
			.wrap_err("failed to get SystemID for HEAD_MOUNTED_DISPLAY form factor")?;
		let hmd_properties = instance
			.system_properties(hmd_system_id)
			.wrap_err("failed to get system properties for HMD")?;
		println!("{hmd_properties:#?}");

		// SAFETY: Headless sessions have no special safety requirements
		if requested_extensions.mnd_headless {
			info!("press enter key to open xr session with headless graphics...");
			std::io::stdin().read_line(&mut String::new()).unwrap();
			let (session, frame_waiter, frame_stream) = unsafe {
				instance.create_session::<Headless>(
					hmd_system_id,
					&HeadlessSessionCreateInfo {},
				)
			}
			.wrap_err("failed to create headless session")?;
			debug!("created headless session");
			do_session(session, frame_waiter, frame_stream)?;
		}

		Ok(())
	}
}

fn do_session<G: Graphics>(
	_session: Session<G>,
	_frame_waiter: FrameWaiter,
	_frame_stream: FrameStream<G>,
) -> Result<()> {
	// TODO: Query devices
	Ok(())
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
