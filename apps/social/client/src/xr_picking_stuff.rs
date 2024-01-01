use bevy::prelude::*;
use bevy_oxr::xr_input::{
	actions::{ActionHandednes, ActionType, SetupActionSets, XrBinding},
	trackers::{
		AimPose, OpenXRController, OpenXRLeftController, OpenXRRightController,
		OpenXRTracker,
	},
};
use picking_xr::{XrActionRef, XrPointer};

pub const ACTION_SET: &str = "picking-nexus";
pub fn spawn_controllers(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
	//left hand
	commands.spawn((
		OpenXRLeftController,
		OpenXRController,
		AimPose(default()),
		XrPointer::new(
			assets.as_mut(),
			picking_xr::XrPickActions {
				primary_action: XrActionRef {
					action_name: "primary_left",
					set_name: ACTION_SET,
				},
				secondary_action: XrActionRef {
					action_name: "secondary_left",
					set_name: ACTION_SET,
				},
				middle_action: XrActionRef {
					action_name: "middle_left",
					set_name: ACTION_SET,
				},
			},
		),
		OpenXRTracker,
		SpatialBundle::default(),
	));
	//right hand
	commands.spawn((
		OpenXRRightController,
		OpenXRController,
		AimPose(default()),
		XrPointer::new(
			assets.as_mut(),
			picking_xr::XrPickActions {
				primary_action: XrActionRef {
					action_name: "primary_right",
					set_name: ACTION_SET,
				},
				secondary_action: XrActionRef {
					action_name: "secondary_right",
					set_name: ACTION_SET,
				},
				middle_action: XrActionRef {
					action_name: "middle_right",
					set_name: ACTION_SET,
				},
			},
		),
		OpenXRTracker,
		SpatialBundle::default(),
	));
}
pub fn setup_xr_actions(mut action_sets: ResMut<SetupActionSets>) {
	let bindings = &[
		XrBinding::new("primary_right", "/user/hand/right/input/trigger/value"),
		XrBinding::new("primary_left", "/user/hand/left/input/trigger/value"),
		XrBinding::new("secondary_right", "/user/hand/right/input/a/click"),
		XrBinding::new("secondary_left", "/user/hand/left/input/x/click"),
		XrBinding::new("middle_right", "/user/hand/right/input/thumbstick/click"),
		XrBinding::new("middle_left", "/user/hand/left/input/thumbstick/click"),
	];
	let set = action_sets.add_action_set(ACTION_SET, "Bevy Mod Picking".into(), 0);
	set.new_action(
		"primary_right",
		"Select Primary Right Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.new_action(
		"primary_left",
		"Select Primary Left Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.new_action(
		"secondary_right",
		"Select Secondary Right Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.new_action(
		"secondary_left",
		"Select Secondary Left Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.new_action(
		"middle_right",
		"Select Middle Right Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.new_action(
		"middle_left",
		"Select Middle Left Hand".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.suggest_binding("/interaction_profiles/oculus/touch_controller", bindings);
}
