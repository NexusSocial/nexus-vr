use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::XrFrameState;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::{QuatConv, Vec3Conv};
use bevy_oxr::DefaultXrPlugins;
use bevy_vrm::{VrmBundle, VrmPlugin};
use color_eyre::Result;
use lightyear::prelude::ClientId;
use social_common::shared::{CLIENT_PORT, SERVER_PORT};
use social_common::Transports;
use std::f32::consts::PI;

use crate::dev_tools::DevToolsPlugins;
use crate::microphone::MicrophonePlugin;

mod dev_tools;
mod humanoid;

mod microphone;
mod networking;

const ASSET_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets/");

#[bevy_main]
pub fn main() {
	color_eyre::install().unwrap();

	let client_id: u16 = random_number::random!();

	info!("Running `social-client`");
	App::new()
		.add_plugins(bevy_web_asset::WebAssetPlugin::default())
		/*.add_plugins(DefaultXrPlugins.set(AssetPlugin {
			file_path: ASSET_FOLDER.to_string(),
			..Default::default()
		}))*/
		.add_plugins(DefaultPlugins)
		.add_plugins(InverseKinematicsPlugin)
		.add_plugins(DevToolsPlugins)
		.add_plugins(VrmPlugin)
		.add_systems(Startup, setup)
		.add_plugins(networking::MyClientPlugin {
			client_id: client_id as u64,
			client_port: portpicker::pick_unused_port().unwrap(),
			server_port: SERVER_PORT,
			transport: Transports::Udp,
		})
		.add_plugins(MicrophonePlugin)
		//.add_systems(Update, hands.map(ignore_on_err))
		.run();
}

pub fn ignore_on_err(_result: Result<()>) {}

pub fn panic_on_err(result: Result<()>) {
	if let Err(err) = result {
		panic!("{err}");
	}
}

pub fn log_on_err(result: Result<()>) {
	if let Err(err) = result {
		error!("{err}");
	}
}

/// set up a simple 3D scene
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	assets: Res<AssetServer>,
) {
	/*let mut transform = Transform::from_xyz(0.0, -1.0, -4.0);
	transform.rotate_y(PI);

	commands.spawn(VrmBundle {
		vrm: assets.load("https://vipe.mypinata.cloud/ipfs/QmU7QeqqVMgnMtCAqZBpAYKSwgcjD4gnx4pxFNY9LqA7KQ/default_398.vrm"),
		scene_bundle: SceneBundle {
			transform,
			..default()
		},
	});*/

	// plane
	commands.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(5.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});
	// cube
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
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
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(-2.0, 2.5, 5.0)
				.looking_at(Vec3::ZERO, Vec3::Y),
			..default()
		},
		bevy_vrm::mtoon::MtoonMainCamera,
	));
	// Avatar
	/*commands.spawn(SceneBundle {
		scene: assets.load("malek.gltf#Scene0"),
		transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_euler(
			EulerRot::XYZ,
			0.0,
			180.0_f32.to_radians(),
			0.0,
		)),
		..default()
	});*/
}

/*fn hands(
	mut gizmos: Gizmos,
	oculus_controller: Res<OculusController>,
	frame_state: Res<XrFrameState>,
	xr_input: Res<XrInput>,
) -> Result<()> {
	let frame_state = *frame_state.lock().unwrap();
	let grip_space = oculus_controller
		.grip_space
		.as_ref()
		.expect("missing grip space on controller");

	let right_controller = grip_space
		.right
		.relate(&xr_input.stage, frame_state.predicted_display_time)?;
	let left_controller = grip_space
		.left
		.relate(&xr_input.stage, frame_state.predicted_display_time)?;
	gizmos.rect(
		right_controller.0.pose.position.to_vec3(),
		right_controller.0.pose.orientation.to_quat(),
		Vec2::new(0.05, 0.2),
		Color::YELLOW_GREEN,
	);
	gizmos.rect(
		left_controller.0.pose.position.to_vec3(),
		left_controller.0.pose.orientation.to_quat(),
		Vec2::new(0.05, 0.2),
		Color::YELLOW_GREEN,
	);
	Ok(())
}
*/
