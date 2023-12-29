#![allow(clippy::type_complexity)]

mod avatars;
mod controllers;
mod microphone;

use bevy::app::PluginGroupBuilder;
use bevy::log::{error, info};
use bevy::pbr::CascadeShadowConfig;
use bevy::prelude::{
	bevy_main, default, shape, Added, App, AssetPlugin, Assets, Camera3dBundle, Color,
	Commands, DirectionalLight, DirectionalLightBundle, Entity, EventWriter, Gizmos,
	Mesh, PbrBundle, PluginGroup, PointLight, PointLightBundle, Quat, Query, Res,
	ResMut, StandardMaterial, Startup, Update, Vec2, Vec3,
};
use bevy::transform::components::Transform;
use bevy::transform::TransformBundle;
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::XrFrameState;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::{QuatConv, Vec3Conv};
use bevy_oxr::DefaultXrPlugins;
use bevy_vrm::VrmPlugin;
use color_eyre::Result;
use std::net::{Ipv4Addr, SocketAddr};

use social_common::dev_tools::DevToolsPlugins;
use social_networking::data_model::Local;
use social_networking::{ClientPlugin, Transports};

use self::avatars::assign::AssignAvatar;
use crate::avatars::{DmEntity, LocalAvatar, LocalEntity};
use crate::microphone::MicrophonePlugin;
// use crate::voice_chat::VoiceChatPlugin;

const ASSET_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets/");

#[bevy_main]
pub fn main() -> Result<()> {
	color_eyre::install()?;

	let mut app = App::new();
	app.add_plugins(bevy_web_asset::WebAssetPlugin)
		.add_plugins(DefaultXrPlugins.set(AssetPlugin {
			file_path: ASSET_FOLDER.to_string(),
			..Default::default()
		}))
		.add_plugins(InverseKinematicsPlugin)
		.add_plugins(DevToolsPlugins)
		.add_plugins(VrmPlugin)
		.add_plugins(NexusPlugins)
		.add_plugins(self::avatars::AvatarsPlugin)
		.add_plugins(self::controllers::KeyboardControllerPlugin)
		.add_systems(Startup, setup)
		.add_systems(Startup, spawn_datamodel_avatar)
		.add_systems(Update, sync_datamodel)
		.add_systems(Startup, try_audio_perms);

	info!("Launching client");
	app.run();
	Ok(())
}

fn try_audio_perms() {
	#[cfg(target_os = "android")]
	{
		request_audio_perm();
	}
}

#[cfg(target_os = "android")]
fn request_audio_perm() {
	let ctx = ndk_context::android_context();
	let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
	let mut env = vm.attach_current_thread().unwrap();
	let activity = unsafe { JObject::from_raw(ctx.context() as jobject) };

	let class_manifest_perm = env.find_class("android/Manifest$permission").unwrap();
	let lid_perm = env
		.get_static_field(class_manifest_perm, "RECORD_AUDIO", "Ljava/lang/String;")
		.unwrap()
		.l()
		.unwrap();

	let classs = env.find_class("java/lang/String").unwrap();
	let inital = env.new_string("").unwrap();
	let perm_list = env.new_object_array(1, classs, inital).unwrap();

	env.set_object_array_element(&perm_list, 0, lid_perm)
		.unwrap();

	let a = JObject::from(perm_list);

	env.call_method(
		activity,
		"requestPermissions",
		"([Ljava/lang/String;I)V",
		&[a.as_ref().into(), JValue::Int(jint::from(1))],
	)
	.unwrap();
}

/// Plugins implemented specifically for this game.
#[derive(Default)]
struct NexusPlugins;

impl PluginGroup for NexusPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(MicrophonePlugin)
			.add(ClientPlugin {
				server_addr: SocketAddr::new(
					Ipv4Addr::new(45, 56, 95, 177).into(),
					social_networking::server::DEFAULT_PORT,
				),
				transport: Transports::Udp,
			})
	}
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

fn sync_datamodel(
	mut cmds: Commands,
	added_avis: Query<
		(
			Entity,
			Option<&Local>,
			Option<&social_networking::Interpolated>,
		),
		Added<social_networking::data_model::Avatar>,
	>,
	mut assign_avi_evts: EventWriter<AssignAvatar>,
) {
	// create our entities the local and verison, when we get a new entity from the
	// network.
	for (dm_avi_entity, local, interp) in added_avis.iter() {
		if local.is_none() && interp.is_none() {
			continue;
		}
		let dm_avi_entity = DmEntity(dm_avi_entity);
		// make sure the data model contains a mapping to the local, and vice versa
		let local_avi_entity = LocalEntity(cmds.spawn(dm_avi_entity).id());
		cmds.entity(dm_avi_entity.0).insert(local_avi_entity);
		// spawn avatar on the local avatar entity
		let avi_url = "https://vipe.mypinata.cloud/ipfs/QmU7QeqqVMgnMtCAqZBpAYKSwgcjD4gnx4pxFNY9LqA7KQ/default_398.vrm".to_owned();
		assign_avi_evts.send(AssignAvatar {
			avi_entity: local_avi_entity.0,
			avi_url,
		});
		if local.is_some() {
			cmds.entity(local_avi_entity.0)
				.insert(LocalAvatar::default());
		} else {
			cmds.entity(local_avi_entity.0)
				.insert(TransformBundle::default());
		}
	}
}

fn spawn_datamodel_avatar(mut cmds: Commands) {
	cmds.spawn((
		social_networking::data_model::Avatar,
		social_networking::data_model::Local,
	));
}

/// set up a simple 3D scene
fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	/*mut assign_avi_evts: EventWriter<AssignAvatar>,*/
) {
	// plane
	cmds.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(5.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});
	// cube
	cmds.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..default()
	});
	// light
	/*cmds.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});*/

	cmds.spawn(DirectionalLightBundle::default());

	// camera
	cmds.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(-2.0, 2.5, 5.0)
				.looking_at(Vec3::ZERO, Vec3::Y),
			..default()
		},
		bevy_vrm::mtoon::MtoonMainCamera,
	));
}

fn _hands(
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

fn pose_gizmo(gizmos: &mut Gizmos, t: &Transform, color: Color) {
	gizmos.ray(t.translation, t.local_x() * 0.1, Color::RED);
	gizmos.ray(t.translation, t.local_y() * 0.1, Color::GREEN);
	gizmos.ray(t.translation, t.local_z() * 0.1, Color::BLUE);
	gizmos.sphere(t.translation, Quat::IDENTITY, 0.1, color);
}
