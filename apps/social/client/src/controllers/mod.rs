use bevy::prelude::*;

use bevy::prelude::GlobalTransform;
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::XrFrameState;
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::trackers::OpenXRTrackingRoot;
use bevy_oxr::xr_input::{QuatConv, Vec3Conv};
use social_networking::data_model::Local;

// use crate::pose_gizmo;

pub struct KeyboardControllerPlugin;

impl Plugin for KeyboardControllerPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(PreUpdate, drive_input_pose);
	}
}

/// Any entity with this attached gets its transform controlled by the keyboard.
#[derive(Component, Debug, Default)]
pub struct KeyboardController;

fn drive_input_pose(
	oculus_controller: Option<Res<OculusController>>,
	frame_state: Option<Res<XrFrameState>>,
	xr_input: Option<Res<XrInput>>,
	tracking_root: Query<&GlobalTransform, With<OpenXRTrackingRoot>>,
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
	let (oculus_controller, frame_state, xr_input) =
		match (oculus_controller, frame_state, xr_input) {
			(Some(oculus_controller), Some(frame_state), Some(xr_input)) => {
				(oculus_controller, frame_state, xr_input)
			}
			(_, _, _) => return,
		};
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

	let headset_input = match xr_input
		.head
		.relate(&xr_input.stage, frame_state.predicted_display_time)
	{
		Ok(h) => h,
		Err(_) => return,
	};

	player_pose.root_scale = tracking_root.to_scale_rotation_translation().0;
	player_pose.root.trans = tracking_root.translation();
	player_pose.root.rot = tracking_root.to_scale_rotation_translation().1;

	player_pose.head.trans = headset_input.0.pose.position.to_vec3();
	player_pose.head.rot = headset_input.0.pose.orientation.to_quat();

	player_pose.hand_r.trans = right_controller.0.pose.position.to_vec3();
	player_pose.hand_r.rot = right_controller.0.pose.orientation.to_quat();

	player_pose.hand_l.trans = left_controller.0.pose.position.to_vec3();
	player_pose.hand_l.rot = left_controller.0.pose.orientation.to_quat();
}
