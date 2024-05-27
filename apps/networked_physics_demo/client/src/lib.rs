use bevy::{
	app::{App, Startup},
	asset::Assets,
	core_pipeline::core_3d::Camera3dBundle,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	ecs::system::{Commands, ResMut},
	log::info,
	math::{
		primitives::{Cuboid, Plane3d},
		Vec3,
	},
	pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	prelude::{bevy_main, default},
	render::{
		color::Color,
		mesh::{Mesh, Meshable},
	},
	transform::components::Transform,
};
use bevy_oxr::DefaultXrPlugins;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[bevy_main]
pub fn main() {
	color_eyre::install().unwrap();

	info!("Running {}", APP_NAME);
	App::new()
		.add_plugins(DefaultXrPlugins {
			app_info: bevy_oxr::graphics::XrAppInfo {
				name: env!("CARGO_PKG_NAME").to_string(),
			},
			..default()
		})
		.add_plugins(LogDiagnosticsPlugin::default())
		.add_plugins(FrameTimeDiagnosticsPlugin)
		.add_systems(Startup, setup)
		// .add_systems(Update, hands)
		.run();
}

// set up a simple 3D scene
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// plane
	commands.spawn(PbrBundle {
		mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(5.0, 5.0)),
		material: materials.add(StandardMaterial::from(Color::rgb(0.3, 0.5, 0.3))),
		..default()
	});
	// cube
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.1)))),
		material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.7, 0.6))),
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..default()
	});
	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});
	// camera
	commands.spawn((Camera3dBundle {
		transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
}
