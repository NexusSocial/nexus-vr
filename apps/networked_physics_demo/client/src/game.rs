//! Main game logic

use std::time::Duration;

use bevy::app::Plugin;
use bevy::ecs::schedule::OnEnter;
use bevy::{
	app::Update,
	asset::{Assets, Handle},
	core::Name,
	core_pipeline::core_3d::Camera3dBundle,
	ecs::{
		component::Component,
		event::{Event, EventReader, EventWriter},
		system::{Commands, Local, Query, Res, ResMut},
	},
	math::{
		primitives::{Cuboid, Plane3d},
		Vec3,
	},
	pbr::{
		light_consts, AmbientLight, DirectionalLight, DirectionalLightBundle,
		PbrBundle, StandardMaterial,
	},
	prelude::default,
	render::{
		camera::{ClearColor, Exposure},
		color::Color,
		mesh::{Mesh, Meshable},
	},
	time::{Time, Timer, TimerMode},
	transform::components::Transform,
};
use bevy_flycam::FlyCam;
use bevy_rapier3d::{
	dynamics::RigidBody,
	geometry::Collider,
	plugin::{NoUserData, RapierPhysicsPlugin},
};
use rand::Rng;

use crate::netcode::LocalAuthority;
use crate::rng::seed_rng;
use crate::{AppExt, GameModeState};

#[derive(Debug, Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_if_not_added(bevy_flycam::NoCameraPlayerPlugin)
			.add_if_not_added(RapierPhysicsPlugin::<NoUserData>::default())
			.add_if_not_added(bevy_flycam::NoCameraPlayerPlugin)
			.add_systems(OnEnter(GameModeState::InMinecraft), setup)
			.add_systems(Update, handle_spawn_cube)
			.add_systems(Update, spawn_cube_timer(Duration::from_millis(1000)))
			.add_event::<SpawnCubeEvt>();
	}
}

// set up a simple 3D scene
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 10000.0,
	});
	commands.insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()));
	// plane
	let normal = Vec3::Y;
	let collider =
		Collider::halfspace(normal).expect("failed to create collider from mesh");
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Plane3d::new(normal).mesh().size(50.0, 50.0).build()),
			material: materials.add(StandardMaterial::from(Color::rgb(0.3, 0.5, 0.3))),
			..default()
		},
		collider,
	));
	// sunlight
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: light_consts::lux::DIRECT_SUNLIGHT,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0.1, 1.0, 0.1).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
	// camera
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(10.0, 2.0, 10.0)
				.looking_at(Vec3::ZERO, Vec3::Y),
			exposure: Exposure::SUNLIGHT,
			..default()
		},
		FlyCam,
	));

	// Cube Spawner
	let cube_side_length = 1.0;
	let mesh = Mesh::from(Cuboid::from_size(Vec3::splat(cube_side_length)));
	let material = materials.add(StandardMaterial::from(Color::WHITE));
	let collider = Collider::cuboid(
		cube_side_length / 2.,
		cube_side_length / 2.,
		cube_side_length / 2.,
	);
	let mesh = meshes.add(mesh);
	commands.spawn((
		CubeSpawner {
			mesh,
			material,
			collider,
		},
		Transform::from_xyz(0.0, 10.0, 0.0),
	));
}

#[derive(Debug, Component)]
pub struct CubeSpawner {
	mesh: Handle<Mesh>,
	material: Handle<StandardMaterial>,
	collider: Collider,
}

#[derive(Event, Default)]
struct SpawnCubeEvt {
	name: Option<String>,
}

seed_rng!(SpawnCubeRng, Ajdckcfupajckvioqweimvidfguaskl);

/// Listens to `SpawnCubeEvt` and spawns a cube at all `CubeSpawner`s.
fn handle_spawn_cube(
	mut rng: Local<SpawnCubeRng>,
	mut cube_num: Local<u64>,
	mut commands: Commands,
	spawner: Query<(&CubeSpawner, &Transform)>,
	mut spawn_cube_evr: EventReader<SpawnCubeEvt>,
) {
	let mut spawn_cube =
		|spawn_cube_evt: &SpawnCubeEvt, spawner: &CubeSpawner, spawn_at| {
			let r = -1.0..=1.0;
			let dx = rng.gen_range(r.clone());
			let dy = rng.gen_range(r.clone());
			let dz = rng.gen_range(r.clone());
			let rand_offset = Transform::from_xyz(dx, dy, dz);
			let mut cube_entity = commands.spawn((
				PbrBundle {
					mesh: spawner.mesh.clone(),
					material: spawner.material.clone(),
					transform: spawn_at * rand_offset,
					..default()
				},
				RigidBody::Dynamic,
				spawner.collider.clone(),
			));
			cube_entity.insert(LocalAuthority);

			if let Some(name) = &spawn_cube_evt.name {
				cube_entity.insert(Name::new(name.clone()));
			} else {
				cube_entity.insert(Name::new(format!("Cube {}", *cube_num)));
			}
			*cube_num += 1;
		};
	for spawn_cube_evt in spawn_cube_evr.read() {
		for (sp, trans) in spawner.iter() {
			spawn_cube(spawn_cube_evt, sp, *trans)
		}
	}
}

/// Sends `SpawnCubeEvt` on a timer.
fn spawn_cube_timer(
	timer_duration: Duration,
) -> impl FnMut(Res<Time>, EventWriter<SpawnCubeEvt>) {
	let mut timer = Timer::new(timer_duration, TimerMode::Repeating);
	move |time, mut spawn_cube_evw| {
		if timer.tick(time.delta()).finished() {
			spawn_cube_evw.send_default();
		}
	}
}
