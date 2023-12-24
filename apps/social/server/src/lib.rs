mod avatars;
mod cli;
mod networking;
mod player_management;
mod voice_chat;

use crate::avatars::Avatars;
use crate::networking::MyServerPlugin;
use crate::player_management::PlayerManagement;
use crate::voice_chat::VoiceChatPlugin;

use bevy::app::PluginGroupBuilder;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::ExitCondition;
use bevy_vrm::VrmPlugin;
use clap::Parser;
use color_eyre::Result;
use social_common::dev_tools::DevToolsPlugins;
use social_common::humanoid::HumanoidPlugin;
use social_common::shared::SERVER_PORT;
use social_common::Transports;

#[bevy_main]
pub fn main() -> Result<()> {
	color_eyre::install()?;

	let mut app = App::new();
	// Must be before default plugins
	app.add_plugins(bevy_web_asset::WebAssetPlugin);

	let args = cli::Cli::parse();

	debug!("headless: {}", args.headless);

	// Core bevy plugins
	match args.headless {
		true => app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>().set(
			WindowPlugin {
				primary_window: None,
				exit_condition: ExitCondition::DontExit,
				close_when_requested: false,
			},
		)),
		false => app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>().set(
			WindowPlugin {
				primary_window: Some(Window {
					title: "Nexus Server".to_string(),
					..Default::default()
				}),
				..Default::default()
			},
		)),
	};

	app.
        // Third party plugins
        add_plugins(VrmPlugin)
		// First party plugins
		.add_plugins(MyServerPlugin {
			port: SERVER_PORT,
			transport: Transports::Udp,
		})
        .add_plugins(Avatars)
        .add_plugins(PlayerManagement)
        .add_plugins(if args.frame_timings {
            DevToolsPlugins.build().disable::<LogDiagnosticsPlugin>()
        } else {
            DevToolsPlugins.build()
        })
		.add_plugins(VoiceChatPlugin);

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
		mesh: meshes.add(shape::Plane::from_size(20.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
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
			.add(MyServerPlugin {
				port: SERVER_PORT,
				transport: Transports::Udp,
			})
			.add(VoiceChatPlugin)
	}
}
