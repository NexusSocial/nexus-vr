use bevy::{
	prelude::{
		Color, Component, Gizmos, Input, KeyCode, Plugin, PreUpdate, Query, Res,
		ResMut, Resource, Transform, Vec3, With,
	},
	reflect::Reflect,
	time::Time,
};

use bevy::prelude::GlobalTransform;
use bevy_oxr::resources::XrFrameState;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::trackers::OpenXRTrackingRoot;
use bevy_oxr::xr_input::{QuatConv, Vec3Conv};
use bevy_oxr::{input::XrInput, resources::XrSession};
use social_networking::data_model::Local;

use crate::pose_gizmo;

const SPEED: f32 = 4.;

pub struct KeyboardControllerPlugin;

impl Plugin for KeyboardControllerPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<Direction>()
			.init_resource::<Direction>()
			.add_systems(PreUpdate, direction_from_keys)
			.add_systems(PreUpdate, move_controlled_entities)
			.add_systems(PreUpdate, drive_input_pose);
	}
}

/// Any entity with this attached gets its transform controlled by the keyboard.
#[derive(Component, Debug, Default)]
pub struct KeyboardController;

/// Direction of movement. Controlled by keyboard.
#[derive(Resource, Debug, PartialEq, Eq, Clone, Default, Reflect)]
struct Direction {
	pub forward: bool,
	pub back: bool,
	pub left: bool,
	pub right: bool,
	pub up: bool,
	pub down: bool,
}

impl Direction {
	pub fn clear(&mut self) {
		self.forward = false;
		self.back = false;
		self.left = false;
		self.right = false;
		self.up = false;
		self.down = false;
	}

	fn velocity(&self) -> Vec3 {
		let mut v = Vec3::ZERO;
		fn f(dir_bool: bool, dir_vec: Vec3) -> Vec3 {
			dir_bool as u8 as f32 * dir_vec
		}

		v += f(self.forward, Vec3::NEG_Z);
		v += f(self.back, Vec3::Z);
		v += f(self.left, Vec3::NEG_X);
		v += f(self.right, Vec3::X);
		v += f(self.up, Vec3::Y);
		v += f(self.down, Vec3::NEG_Y);

		v
	}
}

fn direction_from_keys(
	mut direction: ResMut<Direction>,
	keypress: Res<Input<KeyCode>>,
) {
	direction.clear();
	if keypress.pressed(KeyCode::W) || keypress.pressed(KeyCode::Up) {
		direction.forward = true;
	}
	if keypress.pressed(KeyCode::S) || keypress.pressed(KeyCode::Down) {
		direction.back = true;
	}
	if keypress.pressed(KeyCode::A) || keypress.pressed(KeyCode::Left) {
		direction.left = true;
	}
	if keypress.pressed(KeyCode::D) || keypress.pressed(KeyCode::Right) {
		direction.right = true;
	}
	if keypress.pressed(KeyCode::Space) {
		direction.up = true;
	}
	if keypress.pressed(KeyCode::ShiftLeft) {
		direction.down = true;
	}
}

fn move_controlled_entities(
	direction: Res<Direction>,
	time: Res<Time>,
	mut trans: Query<&mut Transform, With<KeyboardController>>,
	mut gizmos: Gizmos,
) {
	for mut t in trans.iter_mut() {
		let local_vel = direction.velocity() * SPEED * time.delta_seconds();
		let parent_vel = (t.local_x() * local_vel.x)
			+ (t.local_y() * local_vel.y)
			+ (t.local_z() * local_vel.z);
		t.translation += parent_vel;
		pose_gizmo(&mut gizmos, &t, Color::GRAY);
	}
}

fn drive_input_pose(
	mut gizmos: Gizmos,
	oculus_controller: Option<Res<OculusController>>,
	frame_state: Option<Res<XrFrameState>>,
	xr_input: Option<Res<XrInput>>,
	xr_session: Option<Res<XrSession>>,
	tracking_root: Query<&GlobalTransform, &OpenXRTrackingRoot>,
	mut player_pose: Query<&mut social_networking::data_model::PlayerPose, With<Local>>,
) {
	let mut player_pose = match player_pose.get_single_mut() {
		Ok(player_pose) => player_pose,
		Err(_) => return,
	};
	let tracking_root = match tracking_root.get_single() {
		Ok(tracking_root) => tracking_root,
		Err(_) => return,
	};
	let session = match xr_session {
		Some(session) => session,
		None => return,
	};
	let (oculus_controller, frame_state, xr_input) =
		match (oculus_controller, frame_state, xr_input) {
			(Some(oculus_controller), Some(frame_state), Some(xr_input)) => {
				(oculus_controller, frame_state, xr_input)
			}
			(_, _, _) => return,
		};
	let frame_state = *frame_state.lock().unwrap();
	let grip_space = oculus_controller
		.grip_space
		.as_ref()
		.expect("missing grip space on controller");

	let right_controller = match grip_space
		.right
		.relate(&xr_input.stage, frame_state.predicted_display_time)
	{
		Ok(r) => r,
		Err(_) => return,
	};
	let left_controller = match grip_space
		.left
		.relate(&xr_input.stage, frame_state.predicted_display_time)
	{
		Ok(l) => l,
		Err(_) => return,
	};

	use openxr as xr;
	let headset_input = match session
		.create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)
		.unwrap()
		.relate(&xr_input.stage, frame_state.predicted_display_time)
	{
		Ok(h) => h,
		Err(_) => return,
	};
	let root = tracking_root.compute_transform().with_scale(Vec3::ONE);

	player_pose.root.trans = tracking_root.translation();
	player_pose.root.rot = tracking_root.to_scale_rotation_translation().1;

	player_pose.head.trans =
		root.transform_point(headset_input.0.pose.position.to_vec3());
	player_pose.head.rot = tracking_root.to_scale_rotation_translation().1
		* headset_input.0.pose.orientation.to_quat();
	player_pose.hand_r.trans =
		root.transform_point(right_controller.0.pose.position.to_vec3());
	player_pose.hand_r.rot = tracking_root.to_scale_rotation_translation().1
		* right_controller.0.pose.orientation.to_quat();

	player_pose.hand_l.trans =
		root.transform_point(left_controller.0.pose.position.to_vec3());
	player_pose.hand_l.rot = tracking_root.to_scale_rotation_translation().1
		* left_controller.0.pose.orientation.to_quat();
}
