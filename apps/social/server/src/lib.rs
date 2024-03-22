mod avatars;
mod cli;
mod player_management;

use bevy::app::PluginGroupBuilder;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use bevy_vrm::VrmPlugin;
use clap::Parser;
use color_eyre::Result;
use social_common::dev_tools::DevToolsPlugins;
use social_common::humanoid::HumanoidPlugin;
use social_networking::{ServerPlugin, Transports};

#[bevy_main]
pub fn main() -> Result<()> {
	color_eyre::install()?;

	let args = cli::Cli::parse();

	let mut app = App::new();
	// Must be before default plugins
	app.add_plugins(bevy_web_asset::WebAssetPlugin);

	// Core bevy plugins
	debug!("headless: {}", args.headless);
	match args.headless {
		true => {
			app.add_plugins(MinimalPlugins.set(
				bevy::app::ScheduleRunnerPlugin::run_loop(
					std::time::Duration::from_secs_f64(1.0 / 60.0),
				),
			));
		}
		false => {
			app.add_plugins(
				DefaultPlugins
					.build() /* .disable::<LogPlugin>() */
					.set(WindowPlugin {
						primary_window: Some(Window {
							title: "Nexus Server".to_string(),
							..Default::default()
						}),
						..Default::default()
					}),
			);
			app.add_plugins(bevy_egui::EguiPlugin);
			app.add_plugins(VrmPlugin);
			app.add_plugins(if !args.frame_timings {
				DevToolsPlugins.build().disable::<LogDiagnosticsPlugin>()
			} else {
				DevToolsPlugins.build()
			});
		}
	};

	app.
        // Third party plugins
		// First party plugins
        // TODO: migrate to social_networking
		// .add_plugins(MyServerPlugin {
		// 	port: SERVER_PORT,
		// 	transport: Transports::Udp,
		// })
        // TODO: Finish functionality for this plugin and replace the above
		add_plugins(::social_networking::ServerPlugin::default());
	//.add_plugins(Avatars)
	//.add_plugins(PlayerManagement);

	match args.headless {
		true => app.add_systems(Startup, setup_headless),
		false => app.add_systems(Startup, setup),
	};

	info!("Launching server");
	app.run();

	Ok(())
}

fn setup_headless() {}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// camera
	commands.spawn((
		Camera3dBundle {
			projection: OrthographicProjection {
				scale: 3.0,
				scaling_mode: ScalingMode::FixedVertical(2.0),
				..default()
			}
			.into(),
			transform: Transform::from_xyz(5.0, 5.0, 5.0)
				.looking_at(Vec3::ZERO, Vec3::Y),
			..default()
		},
		bevy_vrm::mtoon::MtoonMainCamera,
	));

	// plane
	commands.spawn((PbrBundle {
		mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(20.0, 20.0)),
		material: materials.add(StandardMaterial::from(Color::rgb(0.3, 0.5, 0.3))),
		transform: Transform::from_xyz(0.0, -1.0, 0.0),
		..default()
	},));
	// light
	commands.spawn(PointLightBundle {
		transform: Transform::from_xyz(3.0, 8.0, 5.0),
		..default()
	});
}

/// Plugins implemented specifically for this game.
#[derive(Default)]
struct NexusPlugins;

impl PluginGroup for NexusPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(VrmPlugin)
			.add(HumanoidPlugin)
			.add(ServerPlugin {
				port: social_networking::server::DEFAULT_PORT,
				transport: Transports::Udp,
			})
	}
}
