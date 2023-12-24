mod args;
mod networking;
mod voice_chat;

use crate::networking::MyServerPlugin;
use crate::voice_chat::VoiceChatPlugin;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use bevy::window::ExitCondition;

use bevy_vrm::VrmPlugin;
use social_common::shared::SERVER_PORT;
use social_common::Transports;

fn main() {
	use clap::Parser;
	let mut app = App::new();
	app.add_plugins(bevy_web_asset::WebAssetPlugin);
	let args = args::Args::parse();

	println!("{}", args.headless);

	match args.headless {
		true => app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>().set(
			WindowPlugin {
				primary_window: None,
				exit_condition: ExitCondition::DontExit,
				close_when_requested: false,
			},
		)),
		false => app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>()),
	};
	app.add_plugins(VrmPlugin)
		.add_plugins(MyServerPlugin {
			port: SERVER_PORT,
			transport: Transports::Udp,
		})
		.add_plugins(VoiceChatPlugin);

	match args.headless {
		true => app.add_systems(Startup, setup_headless),
		false => app.add_systems(Startup, setup),
	};

	app.run();
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
