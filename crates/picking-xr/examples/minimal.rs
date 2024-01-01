//! A minimal example of how to add xr picking support to your bevy app.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_oxr::{
	xr_init::XrSetup,
	xr_input::{
		actions::{ActionHandednes, ActionType, SetupActionSets, XrBinding},
		trackers::{
			AimPose, OpenXRController, OpenXRLeftController, OpenXRRightController,
			OpenXRTracker,
		},
	},
	DefaultXrPlugins,
};
use picking_xr::{XrActionRef, XrPickingPlugin, XrPointer};

fn main() {
	App::new()
		.add_plugins(
			DefaultXrPlugins {
				app_info: bevy_oxr::graphics::XrAppInfo {
					name: "Picking Xr Example".to_string(),
				},
				..default()
			}
			.set(low_latency_window_plugin()),
		)
		.add_systems(Startup, spawn_controllers)
		// All you need to do is add the picking plugin, with your backend of choice enabled in the
		// cargo features. By default, the bevy_mod_raycast backend is enabled via the
		// `backend_raycast` feature.
		.add_plugins(DefaultPickingPlugins)
		.add_plugins(XrPickingPlugin)
		.add_systems(Startup, setup)
		.add_systems(XrSetup, setup_xr_actions)
		.run();
}

pub const ACTION_SET: &str = "picking-xr";
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

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
			material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
			..default()
		},
		PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
	));
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
			transform: Transform::from_xyz(0.0, 0.5, 0.0),
			..default()
		},
		PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
	));
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, -4.0),
		..default()
	});
	commands.spawn((Camera3dBundle {
		transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	},));
}
