use crate::file_sharing::FileSharingPlugin;
use crate::networking::{ConnectToRoom, LocalPlayer, NetworkingPlugin, SpawnPlayer};
use crate::player_networking::PlayerNetworking;
use avian3d::prelude::{Collider, CollisionLayers, GravityScale, LockedAxes, RigidBody};
use avian3d::PhysicsPlugins;
use bevy::app::App;
use bevy::asset::{AssetMetaCheck, AssetServer, Assets, Handle};
use bevy::color::Color;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::pbr::{AmbientLight, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_spatial_egui::SpawnSpatialEguiWindowCommand;
use bevy_suis::debug::SuisDebugGizmosPlugin;
use bevy_suis::window_pointers::SuisWindowPointerPlugin;
use bevy_suis::SuisCorePlugin;
use bevy_vr_controller::animation::defaults::default_character_animations;
use bevy_vr_controller::movement::PlayerInputState;
use bevy_vr_controller::player::{
	PlayerAvatar, PlayerBody, PlayerHeight, PlayerJumpHeight, PlayerSettings,
	PlayerSpawn, PlayerSpeed, SpawnedPlayer, VoidTeleport,
};
use bevy_vr_controller::velocity::AverageVelocity;
use bevy_vr_controller::VrControllerPlugin;
use bevy_vrm::VrmBundle;
use uuid::Uuid;
use crate::physics_sync::PhysicsSyncNetworkingPlugin;

mod file_sharing;
pub mod networking;
mod player_networking;
mod physics_sync;

pub fn main() {
	App::new()
		.add_plugins((
			EmbeddedAssetPlugin::default(),
			bevy_web_file_drop::WebFileDropPlugin,
			DefaultPlugins.set(AssetPlugin {
				meta_check: AssetMetaCheck::Never,
				..AssetPlugin::default()
			}),
			PhysicsPlugins::default(),
			VrControllerPlugin,
		))
		.add_plugins((
			SuisCorePlugin,
			SuisWindowPointerPlugin,
			//SuisDebugGizmosPlugin,
		))
		.add_plugins(bevy_spatial_egui::SpatialEguiPlugin)
		.add_plugins(EguiPlugin)
		.add_plugins((NetworkingPlugin, FileSharingPlugin, PlayerNetworking, PhysicsSyncNetworkingPlugin))
		.add_systems(Startup, setup_main_window)
		.add_systems(Startup, (setup_scene, setup_player))
		.add_systems(Update, draw_ui)
		.run();
}

fn draw_ui(mut query: Query<&mut EguiContext, With<MainWindow>>) {
	for mut ctx in &mut query {
		egui::panel::CentralPanel::default().show(ctx.get_mut(), |ui| {
			ui.heading("Hello, World!");
			if ui.button("Press Me!").clicked() {
				info!("Button Pressed");
			}
		});
	}
}

#[derive(Component)]
struct MainWindow;
fn setup_main_window(mut cmds: Commands) {
	let window = cmds.spawn(MainWindow).id();
	cmds.push(SpawnSpatialEguiWindowCommand {
		target_entity: Some(window),
		position: Vec3::new(0.0, 0.5, 0.0),
		rotation: Quat::IDENTITY,
		resolution: UVec2::splat(1024),
		height: 1.0,
		unlit: true,
	});
}

#[derive(Resource)]
pub struct FloorMaterial((Handle<StandardMaterial>, Handle<Mesh>));

const GROUND_SIZE: f32 = 30.0;
const GROUND_THICK: f32 = 0.2;

fn setup_player(asset_server: Res<AssetServer>, mut commands: Commands) {
	let awa = PlayerSettings {
		animations: Some(default_character_animations(&asset_server)),
		vrm: Some(asset_server.load("embedded://models/robot.vrm")),
		void_level: Some(-20.0),
		spawn: Vec3::new(0.0, 3.0, 0.0),
		..default()
	}
	.spawn(&mut commands);
	commands.entity(awa.body).insert(LocalPlayer(Uuid::new_v4()));
}

pub fn spawn_avatar(this: PlayerSettings, commands: &mut Commands) -> Entity {
	let mut body = commands.spawn((
		Collider::capsule(this.width / 2.0, this.height - this.width),
		LockedAxes::ROTATION_LOCKED,
		PlayerBody,
		PlayerSpawn(this.spawn),
		RigidBody::Dynamic,
		SpatialBundle {
			global_transform: GlobalTransform::from_translation(this.spawn),
			..default()
		},
		GravityScale(0.0)
	));

	if let Some(value) = this.void_level {
		body.insert(VoidTeleport(value));
	}

	let body = body.id();

	let mut avatar = commands.spawn((
		PlayerAvatar,
		AverageVelocity {
			target: Some(body),
			..default()
		},
		VrmBundle {
			scene_bundle: SceneBundle {
				transform: Transform::from_xyz(0.0, -this.height / 2.0, 0.0),
				..default()
			},
			vrm: this.vrm.clone().unwrap_or_default(),
			..default()
		},
	));

	if let Some(value) = &this.animations {
		avatar.insert(value.clone());
	}

	let avatar = avatar.id();

	commands.entity(body).push_children(&[avatar]);
	body
}

fn setup_scene(
	mut ambient: ResMut<AmbientLight>,
	mut commands: Commands,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
	asset_server: Res<AssetServer>,
) {
	commands.connect_to_room("my_room_1");

	/*commands.insert_resource(
		bevy::render::render_asset::RenderAssetBytesPerFrame::new(4096),
	);*/

	let floor_texture = asset_server.load("embedded://grass.ktx2");
	let floor_normal_texture = asset_server.load("embedded://grass_normal.ktx2");

	ambient.brightness = 100.0;
	ambient.color = Color::linear_rgb(0.95, 0.95, 1.0);

	commands.spawn(DirectionalLightBundle {
		transform: Transform::from_xyz(4.5, 10.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
	let cube_material = materials.add(Color::linear_rgb(1.0, 0.0, 0.0));

	let box_shape = Cuboid::from_size(Vec3::splat(0.5));
	let box_mesh = meshes.add(box_shape);
	commands.spawn((
		Name::new("Light Box"),
		PbrBundle {
			mesh: box_mesh.clone(),
			material: cube_material.clone(),
			transform: Transform::from_xyz(0.0, 2.0, 3.5),
			..default()
		},
		// All `RigidBody::Dynamic` entities are able to be picked up.
		RigidBody::Dynamic,
		Collider::from(box_shape),
		/*CollisionLayers::new(LAYER_PROPS, LayerMask::ALL),*/
	));

	let floor_material = materials.add(StandardMaterial {
		base_color_texture: Some(floor_texture),
		normal_map_texture: Some(floor_normal_texture),
		emissive_exposure_weight: 0.0,
		perceptual_roughness: 1.0,
		reflectance: 0.2,
		specular_transmission: 0.0,
		diffuse_transmission: 0.0,
		thickness: 0.0,
		ior: 1.0,
		clearcoat: 0.0,
		anisotropy_strength: 0.0,
		lightmap_exposure: 1.0,
		..default()
	});

	let floor_mesh = meshes.add(Mesh::from(Cuboid::new(
		GROUND_SIZE / FLOOR_TILING as f32,
		GROUND_THICK,
		GROUND_SIZE / FLOOR_TILING as f32,
	)));

	commands
		.insert_resource(FloorMaterial((floor_material.clone(), floor_mesh.clone())));

	const FLOOR_TILING: i32 = 20;

	for x in -FLOOR_TILING..FLOOR_TILING {
		for y in -FLOOR_TILING..FLOOR_TILING {
			commands.spawn((
				PbrBundle {
					mesh: floor_mesh.clone_weak(),
					material: floor_material.clone_weak(),
					transform: Transform::from_xyz(
						(GROUND_SIZE / FLOOR_TILING as f32) * x as f32,
						-1.0 - GROUND_THICK / 2.0,
						(GROUND_SIZE / FLOOR_TILING as f32) * y as f32,
					),
					..default()
				},
				RigidBody::Static,
				Collider::cuboid(
					(GROUND_SIZE / FLOOR_TILING as f32) - 0.01,
					GROUND_THICK,
					(GROUND_SIZE / FLOOR_TILING as f32) - 0.01,
				),
			));
		}
	}

	let mut transform = Transform::from_xyz(0.0, 3.0, -10.0);
	transform.look_at(Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 1.0, 0.0));
}
