//! Demo Bevy OpenXR VR application.
use bevy::{
	DefaultPlugins,
	app::{App, AppExit, Startup},
	asset::Assets,
	camera::Camera3d,
	color::Color,
	ecs::system::{Commands, ResMut},
	light::PointLight,
	math::{
		Quat, Vec3,
		primitives::{Circle, Cuboid, Cylinder},
	},
	mesh::{Mesh, Mesh3d},
	pbr::{MeshMaterial3d, StandardMaterial},
	transform::components::Transform,
	utils::default,
};
use color_eyre::{Result, eyre::bail};

fn main() -> Result<()> {
	color_eyre::install()?;

	let mut app = App::new();
	app.add_plugins(DefaultPlugins).add_systems(Startup, setup);

	if let AppExit::Error(code) = app.run() {
		bail!("bevy app exited with error code {code}")
	};

	Ok(())
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// Ground plane
	commands.spawn((
		Mesh3d(meshes.add(Circle::new(4.0))),
		MeshMaterial3d(materials.add(Color::WHITE)),
		Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
	));

	// A few cubes to look at
	let colors = [
		Color::srgb(1.0, 0.3, 0.3),
		Color::srgb(0.3, 1.0, 0.3),
		Color::srgb(0.3, 0.3, 1.0),
		Color::srgb(1.0, 1.0, 0.3),
	];

	for (i, color) in colors.iter().enumerate() {
		let angle = std::f32::consts::TAU * (i as f32) / (colors.len() as f32);
		let x = angle.cos() * 2.0;
		let z = angle.sin() * 2.0;

		commands.spawn((
			Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
			MeshMaterial3d(materials.add(*color)),
			Transform::from_xyz(x, 0.5, z),
		));
	}

	// Central pedestal
	commands.spawn((
		Mesh3d(meshes.add(Cylinder::new(0.3, 1.0))),
		MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
		Transform::from_xyz(0.0, 0.5, 0.0),
	));

	// Light
	commands.spawn((
		PointLight {
			shadows_enabled: true,
			intensity: 2_000_000.0,
			..default()
		},
		Transform::from_xyz(4.0, 8.0, 4.0),
	));

	// Fallback camera (for non-VR testing)
	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
	));
}
