//! This module is responsible for displaying an Entity Inspector in the world, so Vr Users can
//! interact with it. this inspector is not synced over the network, and only one can exist at a
//! time.

use std::f32::consts::{PI, TAU};

use bevy::{
	prelude::*,
	render::render_resource::{Extent3d, TextureUsages},
};
use bevy_oxr::{
	resources::XrSession,
	xr_init::{xr_only, XrSetup},
	xr_input::{
		actions::{
			ActionHandednes, ActionType, SetupActionSets, XrActionSets, XrBinding,
		},
		trackers::{AimPose, OpenXRLeftController, OpenXRTrackingRoot},
	},
};
use egui_picking::WorldSpaceUI;
use social_common::dev_tools::InspectorUiRenderTarget;

use crate::LegalStdMaterial;
use bevy_inspector_egui::{
	inspector_options::ReflectInspectorOptions, InspectorOptions,
};

#[derive(Default)]
pub struct InWorldInspectorPlugin;
impl Plugin for InWorldInspectorPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(XrSetup, setup_xr_actions);
		app.add_systems(Update, toggle_inspector.run_if(xr_only()));
		app.add_systems(Startup, new_inspector_texture);
		app.register_type::<InWorldInspector>();
		app.register_type::<InWorldInspectorTexture>();
	}
}

fn new_inspector_texture(mut cmds: Commands, mut images: ResMut<Assets<Image>>) {
	let output_texture = images.add({
		let size = Extent3d {
			width: 450,
			height: 450,
			depth_or_array_layers: 1,
		};
		let mut output_texture = Image {
			data: vec![0; (size.width * size.height * 4) as usize],
			..default()
		};
		output_texture.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
		output_texture.texture_descriptor.size = size;
		output_texture
	});
	cmds.insert_resource(InWorldInspectorTexture(output_texture));
}

#[derive(Default, Clone, Copy, Debug, Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct InWorldInspector;

#[derive(Default, Clone, Debug, Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct InWorldInspectorTexture(Handle<Image>);

const ACTION_SET: &str = "inspector";
// TODO: Make work with keyboard/gamepad
fn toggle_inspector(
	mut cmds: Commands,
	in_world_inspectors: Query<Entity, With<InWorldInspector>>,
	texture: Res<InWorldInspectorTexture>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	action_sets: Res<XrActionSets>,
	session: Res<XrSession>,
	aim_pose: Query<&AimPose, With<OpenXRLeftController>>,
	root: Query<&Transform, With<OpenXRTrackingRoot>>,
	mut last_pressed: Local<bool>,
) {
	let now_pressed = action_sets
		.get_action_bool(ACTION_SET, "toggle")
		.expect("Action has to exist and be of the correct type")
		.state(&session, openxr::Path::NULL)
		.expect("Something went horribly wrong, the actionset is not attached")
		.current_state;

	if now_pressed && !*last_pressed {
		if !in_world_inspectors.is_empty() {
			for e in &in_world_inspectors {
				cmds.entity(e).despawn();
			}
		} else {
			let transform = root.single().mul_transform(aim_pose.single().0);
			let mut plane_transform = Transform::from_translation(
				transform.translation + transform.forward(),
			)
			.looking_at(transform.translation, Vec3::Y);
			plane_transform.rotate_local(
				Quat::from_rotation_x(TAU * -1.0 / 4.0) * Quat::from_rotation_y(PI),
			);
			cmds.spawn((
				PbrBundle {
					mesh: meshes.add(shape::Plane::default().into()),
					material: materials.add(StandardMaterial {
						base_color: Color::WHITE,
						base_color_texture: Some(Handle::clone(&texture.0)),
						unlit: true,
						..default()
					}),
					transform: plane_transform,
					..default()
				},
				WorldSpaceUI::new(Handle::clone(&texture.0), 1.0, 1.0),
				InspectorUiRenderTarget,
				InWorldInspector,
				LegalStdMaterial,
				Name::new("Inspector"),
			));
		}
	}
	*last_pressed = now_pressed;
}

fn setup_xr_actions(mut action_sets: ResMut<SetupActionSets>) {
	let bindings = &[XrBinding::new("toggle", "/user/hand/left/input/menu/click")];
	let set = action_sets.add_action_set(ACTION_SET, "In World Inspector".into(), 0);
	set.new_action(
		"toggle",
		"Open/Close Inspector".into(),
		ActionType::Bool,
		ActionHandednes::Single,
	);
	set.suggest_binding("/interaction_profiles/oculus/touch_controller", bindings);
}
