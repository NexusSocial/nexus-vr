#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod avatars;
mod controllers;
mod custom_audio;
mod ik;
mod voice_chat;

use bevy::app::PluginGroupBuilder;
use bevy::asset::Handle;
use bevy::core::Name;
use bevy::ecs::query::{With, Without};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::log::{error, info};
use bevy::pbr::{AmbientLight, DirectionalLight, PointLight};
use bevy::prelude::{
	bevy_main, default, Added, App, AssetPlugin, Assets, Camera3dBundle, Color,
	Commands, DirectionalLightBundle, Entity, EventWriter, Gizmos, PluginGroup, Quat,
	Query, Res, ResMut, StandardMaterial, Startup, Update, Vec2, Vec3,
};
use bevy::scene::SceneBundle;
use bevy::transform::components::Transform;
use bevy::transform::TransformBundle;
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::XrFrameState;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::trackers::OpenXRRightEye;
use bevy_oxr::xr_input::{QuatConv, Vec3Conv};
use bevy_oxr::DefaultXrPlugins;
use bevy_vrm::mtoon::{MtoonMainCamera, MtoonMaterial};
use bevy_vrm::VrmPlugin;
use color_eyre::Result;
use rodio::SpatialSink;
use std::net::{Ipv4Addr, SocketAddr};

use social_common::dev_tools::DevToolsPlugins;
use social_networking::data_model::{ClientIdComponent, Local};
use social_networking::{ClientPlugin, Transports};

use self::avatars::assign::AssignAvatar;
use self::ik::IKPlugin;
use crate::avatars::{DmEntity, LocalAvatar, LocalEntity};
use crate::custom_audio::audio_output::AudioOutput;
use crate::custom_audio::spatial_audio::SpatialAudioSink;

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
		.add_plugins(IKPlugin)
		.add_plugins(DevToolsPlugins)
		.add_plugins(VrmPlugin)
		.add_plugins(NexusPlugins)
		.add_plugins(self::avatars::AvatarsPlugin)
		.add_plugins(self::controllers::KeyboardControllerPlugin)
		.add_plugins(self::custom_audio::CustomAudioPlugins)
		.add_plugins(self::voice_chat::VoiceChatPlugin)
		.add_systems(Startup, setup)
		.add_systems(Startup, spawn_datamodel_avatar)
		.add_systems(Update, sync_datamodel)
		.add_systems(Startup, try_audio_perms)
		.add_systems(Update, nuke_standard_material)
		.add_systems(Update, vr_rimlight)
		.add_systems(Update, going_dark);
	// .add_systems(Startup, spawn_test_audio)
	// .add_systems(Update, test_loopback_audio);

	info!("Launching client");
	app.run();
	Ok(())
}

// fn spawn_test_audio(mut commands: Commands, mut audio_output: ResMut<AudioOutput>) {
// 	commands.spawn(SpatialAudioSinkBundle {
// 		spatial_audio_sink: SpatialAudioSink {
// 			sink: SpatialSink::try_new(
// 				audio_output.stream_handle.as_ref().unwrap(),
// 				[0.0, 0.0, 0.0],
// 				(Vec3::X * 4.0 / -2.0).to_array(),
// 				(Vec3::X * 4.0 / 2.0).to_array(),
// 			)
// 			.unwrap(),
// 		},
// 		spatial_bundle: SpatialBundle {
// 			transform: Transform::from_xyz(1.0, 0.0, 0.0),
// 			..default()
// 		},
// 	});
// 	// commands.spawn(SpatialAudioListenerBundle { spatial_audio_listener: SpatialAudioListener, spatial_bundle: Default::default() });
// }
// fn test_loopback_audio(mut microphone: ResMut<MicrophoneAudio>, spatial_audio_sink: Query<&SpatialAudioSink>) {
// 	let spatial_audio_sink = spatial_audio_sink.single();
// 	while let Ok( audio) = microphone.0.lock().unwrap().try_recv() {
// 		spatial_audio_sink.sink.append(rodio::buffer::SamplesBuffer::new(1, 44100, audio));
// 		spatial_audio_sink.sink.set_volume(1.0);
// 		spatial_audio_sink.sink.set_speed(1.0);
// 		spatial_audio_sink.sink.play();
// 	}
// }

fn try_audio_perms() {
	#[cfg(target_os = "android")]
	{
		request_audio_perm();
	}
}

#[cfg(target_os = "android")]
fn request_audio_perm() {
	use jni::sys::jobject;

	let ctx = ndk_context::android_context();
	let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
	let mut env = vm.attach_current_thread().unwrap();
	let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as jobject) };

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

	let a = jni::objects::JObject::from(perm_list);

	env.call_method(
		activity,
		"requestPermissions",
		"([Ljava/lang/String;I)V",
		&[
			a.as_ref().into(),
			jni::objects::JValue::Int(jni::sys::jint::from(1)),
		],
	)
	.unwrap();
}

/// Plugins implemented specifically for this game.
#[derive(Default)]
struct NexusPlugins;

impl PluginGroup for NexusPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>().add(ClientPlugin {
			server_addr: SocketAddr::new(
				Ipv4Addr::new(45, 79, 142, 199).into(),
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
	audio_output: ResMut<AudioOutput>,
	added_avis: Query<
		(
			Entity,
			&ClientIdComponent,
			Option<&Local>,
			Option<&social_networking::Interpolated>,
		),
		Added<social_networking::data_model::ClientIdComponent>,
	>,
	mut assign_avi_evts: EventWriter<AssignAvatar>,
) {
	// create our entities the local and verison, when we get a new entity from the
	// network.
	for (dm_avi_entity, client_id_component, local, interp) in added_avis.iter() {
		if local.is_none() && interp.is_none() {
			continue;
		}
		let dm_avi_entity = DmEntity(dm_avi_entity);
		// make sure the data model contains a mapping to the local, and vice versa
		let local_avi_entity = LocalEntity(cmds.spawn(dm_avi_entity).id());
		cmds.entity(dm_avi_entity.0).insert(local_avi_entity);
		// spawn avatar on the local avatar entity
		let avi_url = "https://cdn.discordapp.com/attachments/1190761425396830369/1190863195418677359/malek.vrm".to_owned();
		assign_avi_evts.send(AssignAvatar {
			avi_entity: local_avi_entity.0,
			avi_url,
		});
		if local.is_some() {
			cmds.entity(local_avi_entity.0)
				.insert(LocalAvatar::default());
			cmds.entity(local_avi_entity.0).insert(SpatialAudioSink {
				sink: SpatialSink::try_new(
					audio_output.stream_handle.as_ref().unwrap(),
					[0.0, 0.0, 0.0],
					(Vec3::X * 4.0 / -2.0).to_array(),
					(Vec3::X * 4.0 / 2.0).to_array(),
				)
				.unwrap(),
			});
			cmds.entity(local_avi_entity.0)
				.insert(client_id_component.clone());
		} else {
			cmds.entity(local_avi_entity.0)
				.insert(TransformBundle::default());
			cmds.entity(local_avi_entity.0).insert(SpatialAudioSink {
				sink: SpatialSink::try_new(
					audio_output.stream_handle.as_ref().unwrap(),
					[0.0, 0.0, 0.0],
					(Vec3::X * 4.0 / -2.0).to_array(),
					(Vec3::X * 4.0 / 2.0).to_array(),
				)
				.unwrap(),
			});
			cmds.entity(local_avi_entity.0)
				.insert(client_id_component.clone());
		}
	}
}

fn spawn_datamodel_avatar(mut cmds: Commands) {
	cmds.spawn((
		social_networking::data_model::Avatar,
		social_networking::data_model::Local,
	));
}

fn going_dark(
	mut cmds: Commands,
	cam: Query<(Entity, &Name)>,
	dir_light: Query<Entity, With<DirectionalLight>>,
	point_light: Query<Entity, With<PointLight>>,
) {
	for e in &dir_light {
		cmds.entity(e).despawn_recursive();
	}
	for e in &point_light {
		cmds.entity(e).despawn_recursive();
	}
	for (e, name) in &cam {
		if name.as_str() == "Main Camera" {
			info!("Cam: {:?}", e);
			cmds.entity(e).despawn_recursive();
		}
	}
}

fn nuke_standard_material(
	mut cmds: Commands,
	std_shaders: ResMut<Assets<StandardMaterial>>,
	mut mtoon_shaders: ResMut<Assets<MtoonMaterial>>,
	query: Query<(Entity, &Handle<StandardMaterial>)>,
) {
	for (e, handle) in &query {
		info!("Nuking: {:?}", e);
		let mtoon = match std_shaders.get(handle) {
			Some(shader) => MtoonMaterial {
				base_color: shader.base_color,
				base_color_texture: shader.base_color_texture.clone(),
				rim_lighting_mix_factor: 0.0,
				parametric_rim_color: Color::BLACK,
				parametric_rim_lift_factor: 0.0,
				parametric_rim_fresnel_power: 0.0,
				// shade_color: Color::DARK_GRAY,
				// light_color: Color::GRAY,
				// shading_toony_factor: 0.0,
				// shading_shift_factor: 0.0,
				// light_dir: Vec3::ZERO,
				..default()
			},
			None => MtoonMaterial {
				rim_lighting_mix_factor: 0.0,
				parametric_rim_color: Color::BLACK,
				parametric_rim_lift_factor: 0.0,
				parametric_rim_fresnel_power: 0.0,
				..default()
			},
		};
		let handle = mtoon_shaders.add(mtoon);
		cmds.entity(e)
			.remove::<Handle<StandardMaterial>>()
			.insert(handle);
	}
}

fn vr_rimlight(
	mut cmds: Commands,
	query: Query<Entity, (With<OpenXRRightEye>, Without<MtoonMainCamera>)>,
	query2: Query<Entity, With<MtoonMainCamera>>,
) {
	for e in &query {
		info!("new bevy_mtoon cam: {:?}", e);
		cmds.entity(e).insert(MtoonMainCamera);
		for e in &query2 {
			info!("Murdering {:?}", e);
			cmds.entity(e).remove::<MtoonMainCamera>();
		}
	}
}

/// set up a simple 3D scene
fn setup(
	mut cmds: Commands,
	// mut meshes: ResMut<Assets<Mesh>>,
	// mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: ResMut<bevy::prelude::AssetServer>,
	/*mut assign_avi_evts: EventWriter<AssignAvatar>,*/
) {
	cmds.insert_resource(AmbientLight {
		color: Color::BLACK,
		brightness: 0.0,
	});
	cmds.spawn(SceneBundle {
		scene: asset_server.load("https://cdn.discordapp.com/attachments/1122267109376925738/1190479732819623936/SGB_tutorial_scene_fixed.glb#Scene0"),
		..default()
	});
	// // plane
	// cmds.spawn(PbrBundle {
	// 	mesh: meshes.add(shape::Plane::from_size(5.0).into()),
	// 	material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
	// 	..default()
	// });
	// // cube
	// cmds.spawn(PbrBundle {
	// 	mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
	// 	material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
	// 	transform: Transform::from_xyz(0.0, 0.5, 0.0),
	// 	..default()
	// });
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
