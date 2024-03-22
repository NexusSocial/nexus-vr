use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_inspector_egui::{
	inspector_options::ReflectInspectorOptions, InspectorOptions,
};
use bevy_oxr::{
	input::XrInput,
	resources::XrFrameState,
	xr_init::{xr_only, XrPostSetup, XrSetup},
	xr_input::{trackers::OpenXRTrackingRoot, QuatConv, Vec3Conv},
};
use bevy_schminput::{mouse::MouseBindings, mouse_action, prelude::*, ActionTrait};
use bevy_vrm::mtoon::MtoonMainCamera;

use crate::cursor_lock::LockingMouseActionPlugin;

pub struct MovementPugin;

impl Plugin for MovementPugin {
	fn build(&self, app: &mut App) {
		app.register_type::<MovementForwardRef>();
		app.register_type::<MovementRotate>();
		app.register_type::<MovementControlled>();
		app.register_type::<MovementRotatePitch>();
		app.register_type::<MovementRotateYaw>();
		app.register_type::<PlayerRoot>();
		app.register_action::<MoveAction>();
		app.register_action::<ViewAction>();
		app.add_plugins(LockingMouseActionPlugin::<ViewAction>::default());
		app.add_systems(Update, move_entities);
		app.add_systems(Update, drive_non_xr_input_pose);
		app.add_systems(Update, rotate_entities.before(move_entities));
		app.add_systems(Startup, setup_actions);
		app.add_systems(Startup, spawn_player);
		app.add_systems(XrPostSetup, spawn_player);
		app.add_systems(Update, vr_player_head_update.run_if(xr_only()));
	}
}

fn vr_player_head_update(
	mut player_heads: Query<(&Parent, &mut Transform), With<PlayerHead>>,
	root: Query<Has<OpenXRTrackingRoot>>,
	frame_state: Res<XrFrameState>,
	xr_input: Res<XrInput>,
) {
	for (parent, mut transform) in &mut player_heads {
		if let Ok(true) = root.get(parent.get()) {
			let t = match xr_input
				.head
				.relate(&xr_input.stage, frame_state.predicted_display_time)
			{
				Ok(t) => t,
				Err(_) => continue,
			};
			transform.translation = t.0.pose.position.to_vec3();
			transform.rotation = t.0.pose.orientation.to_quat();
		};
	}
}

fn spawn_player(
	mut cmds: Commands,
	root_query: Query<Entity, With<OpenXRTrackingRoot>>,
	old_query: Query<Entity, Or<(With<PlayerHead>, With<PlayerRoot>)>>,
) {
	for e in &old_query {
		cmds.entity(e).despawn_recursive();
	}

	if let Ok(root) = root_query.get_single() {
		let e = cmds
			.spawn((
				PlayerHead,
				Camera3dBundle {
					projection: Projection::Perspective(PerspectiveProjection {
						fov: TAU / 6.0,
						..default()
					}),
					..default()
				},
			))
			.id();
		cmds.entity(root).push_children(&[e]);
		cmds.entity(root).insert((
			MovementRotateYaw,
			MovementControlled,
			PlayerRoot,
			MovementRotate,
			MovementRotateAround(e),
			MovementForwardRef(e),
		));
	} else {
		let cam = cmds
			.spawn((
				Camera3dBundle {
					projection: Projection::Perspective(PerspectiveProjection {
						fov: TAU / 6.0,
						..default()
					}),
					transform: Transform::from_xyz(
						0.0,
						1.69 / 1.10, /*yoinked form ik*/
						0.0,
					),
					..default()
				},
				MovementRotate,
				MovementRotatePitch,
				MtoonMainCamera,
				PlayerHead,
			))
			.id();
		cmds.spawn((
			SpatialBundle::default(),
			MovementRotateYaw,
			MovementControlled,
			MovementRotate,
			PlayerRoot,
			MovementForwardRef(cam),
		))
		.push_children(&[cam]);
	}
}

fn rotate_entities(
	action_query: Query<&ViewAction>,
	global_transforms: Query<&GlobalTransform>,
	mut query: Query<
		(
			&mut Transform,
			Option<&MovementRotateAround>,
			Has<MovementRotatePitch>,
			Has<MovementRotateYaw>,
		),
		With<MovementRotate>,
	>,
	time: Res<Time>,
) {
	for (mut transform, around, pitch, yaw) in &mut query {
		let action = match action_query.get_single() {
			Ok(v) => v,
			Err(_) => continue,
		};
		if yaw {
			match around.and_then(|e| global_transforms.get(e.0).ok()) {
				Some(around) => transform.rotate_around(
					around.translation(),
					Quat::from_rotation_y(
						-action.get_value().x * TAU * time.delta_seconds(),
					),
				),
				None => transform
					.rotate_y(-action.get_value().x * TAU * time.delta_seconds()),
			};
		}
		if pitch {
			transform
				.rotate_local_x(-action.get_value().y * TAU * time.delta_seconds());
			let (pitch, _yaw, _roll) = transform.rotation.to_euler(EulerRot::XYZ);
			if pitch > TAU / 4.0 {
				transform.rotate_local_x(-(pitch - TAU / 4.0) - 0.00001)
			}
			if pitch < -(TAU / 4.0) {
				transform.rotate_local_x(-(pitch + TAU / 4.0) + 0.00001)
			}
		}
	}
}

fn setup_actions(
	mut cmds: Commands,
	mut oxr: ResMut<OXRSetupBindings>,
	mut keyb: ResMut<KeyboardBindings>,
	mut mouse: ResMut<MouseBindings>,
) {
	let move_action = MoveAction::default();
	let view_action = ViewAction {
		mouse_sens_x: 0.1,
		mouse_sens_y: 0.1,
		..default()
	};
	oxr.add_binding(
		&move_action,
		OXRBinding {
			device: "/interaction_profiles/oculus/touch_controller",
			binding: "/user/hand/left/input/thumbstick",
		},
	);
	oxr.add_binding(
		&view_action,
		OXRBinding {
			device: "/interaction_profiles/oculus/touch_controller",
			binding: "/user/hand/right/input/thumbstick",
		},
	);
	mouse.add_motion_binding(&view_action);
	keyb.add_binding(
		&move_action,
		KeyboardBinding::Dpad {
			up: KeyBinding::Held(KeyCode::KeyW),
			down: KeyBinding::Held(KeyCode::KeyS),
			left: KeyBinding::Held(KeyCode::KeyA),
			right: KeyBinding::Held(KeyCode::KeyD),
		},
	);
	keyb.add_binding(
		&view_action,
		KeyboardBinding::Dpad {
			up: KeyBinding::Held(KeyCode::ArrowUp),
			down: KeyBinding::Held(KeyCode::ArrowDown),
			left: KeyBinding::Held(KeyCode::ArrowLeft),
			right: KeyBinding::Held(KeyCode::ArrowRight),
		},
	);
	cmds.spawn((move_action, view_action));
}

struct MovementActionSet;
impl MovementActionSet {
	fn name() -> String {
		"Movement".into()
	}
	fn key() -> &'static str {
		"movement"
	}
}

basic_action!(
	MoveAction,
	Vec2,
	"move",
	"Move".into(),
	MovementActionSet::key(),
	MovementActionSet::name()
);

mouse_action!(
	ViewAction,
	"view",
	"View Input".into(),
	MovementActionSet::key(),
	MovementActionSet::name()
);

fn move_entities(
	forward_query: Query<&GlobalTransform>,
	mut moving_entity_query: Query<
		(&mut Transform, Option<&MovementForwardRef>),
		With<MovementControlled>,
	>,
	move_action: Query<&MoveAction>,
	time: Res<Time>,
) {
	for (mut transform, forward_ref) in &mut moving_entity_query {
		info!("moving_loop");
		let forward_rot = forward_ref
			.and_then(|r| forward_query.get(**r).ok())
			.map(|f| f.to_scale_rotation_translation().1)
			.unwrap_or(transform.rotation);
		let (yaw, _pitch, _roll) = forward_rot.to_euler(EulerRot::YXZ);
		let forward_quat = Quat::from_axis_angle(Vec3::Y, yaw);
		let input_vec = match move_action.get_single() {
			Ok(v) => v,
			Err(_) => continue,
		}
		.get_value();

		info!("moving_values: {}", input_vec);
		let mut in_vec = Vec3::new(input_vec.x, 0.0, -input_vec.y);
		if in_vec.length() > 1.0 {
			in_vec = in_vec.normalize_or_zero();
		}
		transform.translation +=
			forward_quat.mul_vec3(in_vec * time.delta_seconds() * 4.0);
	}
}

#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct PlayerRoot;
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct PlayerHead;
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementForwardRef(Entity);
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementControlled;
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementRotate;
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementRotatePitch;
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementRotateYaw;
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementRotateAround(Entity);

fn drive_non_xr_input_pose(
	tracking_root: Query<(&GlobalTransform, Entity), With<PlayerRoot>>,
	children: Query<&Children>,
	head_query: Query<&Transform, With<PlayerHead>>,
	mut player_pose: Query<
		&mut social_networking::data_model::PlayerPose,
		With<social_networking::data_model::Local>,
	>,
) {
	let mut player_pose = match player_pose.get_single_mut() {
		Ok(player_pose) => player_pose,
		Err(_) => return,
	};
	let (tracking_root, root) = match tracking_root.get_single() {
		Ok(tracking_root) => tracking_root,
		Err(_) => return,
	};
	let mut head = None;
	for e in children.iter_descendants(root) {
		if let Ok(h) = head_query.get(e) {
			head = Some(h);
			break;
		}
	}
	let head = match head {
		Some(h) => h,
		None => return,
	};
	player_pose.root_scale = tracking_root.to_scale_rotation_translation().0;
	player_pose.root.trans = tracking_root.translation();
	player_pose.root.rot = tracking_root.to_scale_rotation_translation().1;

	player_pose.head.trans = head.translation;
	player_pose.head.rot = head.rotation;

	player_pose.hand_r.trans = Vec3::new(0.25, 0.8, 0.0);
	player_pose.hand_r.rot = Quat::IDENTITY;

	player_pose.hand_l.trans = Vec3::new(-0.25, 0.8, 0.0);
	player_pose.hand_l.rot = Quat::IDENTITY;
}
