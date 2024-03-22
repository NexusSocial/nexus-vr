//! This module is responsible for displaying an Entity Inspector in the world, so Vr Users can
//! interact with it. this inspector is not synced over the network, and only one can exist at a
//! time.

use bevy_schminput::{prelude::*, ActionTrait};
use std::f32::consts::{PI, TAU};

use bevy::{
	prelude::*,
	render::render_resource::{Extent3d, TextureUsages},
};
use bevy_oxr::{
	xr_init::XrStatus,
	xr_input::trackers::{AimPose, OpenXRLeftController},
};
use egui_picking::WorldSpaceUI;
use social_common::dev_tools::InspectorUiRenderTarget;

use crate::{
	movement::{PlayerHead, PlayerRoot},
	LegalStdMaterial,
};
use bevy_inspector_egui::{
	inspector_options::ReflectInspectorOptions, InspectorOptions,
};

#[derive(Default)]
pub struct InWorldInspectorPlugin;
impl Plugin for InWorldInspectorPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_actions);
		app.add_systems(Update, toggle_inspector);
		app.add_systems(Startup, new_inspector_texture);
		app.register_type::<InWorldInspector>();
		app.register_type::<InWorldInspectorTexture>();
		app.register_action::<OpenInspector>();
	}
}
fn setup_actions(
	mut cmds: Commands,
	mut oxr: ResMut<OXRSetupBindings>,
	mut keyb: ResMut<KeyboardBindings>,
) {
	let open_inspector = OpenInspector::default();
	oxr.add_binding(
		&open_inspector,
		OXRBinding {
			device: "/interaction_profiles/oculus/touch_controller",
			binding: "/user/hand/left/input/menu/click",
		},
	);
	oxr.add_binding(
		&open_inspector,
		OXRBinding {
			device: "/interaction_profiles/oculus/touch_controller",
			binding: "/user/hand/left/input/y/click",
		},
	);
	keyb.add_binding(
		&open_inspector,
		KeyboardBinding::Simple(KeyBinding::JustPressed(KeyCode::KeyQ)),
	);
	cmds.spawn(open_inspector);
}

basic_action!(
	OpenInspector,
	bool,
	"open",
	"Open Inspector".into(),
	InspectorActionSet::key(),
	InspectorActionSet::name()
);

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
pub struct InspectorActionSet;
impl InspectorActionSet {
	pub fn key() -> &'static str {
		"inspector"
	}
	pub fn name() -> String {
		"Inspector".into()
	}
}
// TODO: Make work with keyboard/gamepad
fn toggle_inspector(
	mut cmds: Commands,
	in_world_inspectors: Query<Entity, With<InWorldInspector>>,
	texture: Res<InWorldInspectorTexture>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	aim_pose: Query<&AimPose, With<OpenXRLeftController>>,
	cam_pose: Query<&Transform, With<PlayerHead>>,
	root: Query<&Transform, With<PlayerRoot>>,
	open_action: Query<&OpenInspector>,
	xr_enabled: Option<Res<XrStatus>>,
	mut last_pressed: Local<bool>,
) {
	let pressed = match open_action.get_single() {
		Ok(v) => *v.get_value(),
		Err(_) => return,
	};
	if pressed && !*last_pressed {
		if !in_world_inspectors.is_empty() {
			for e in &in_world_inspectors {
				cmds.entity(e).despawn();
			}
		} else {
			let transform = match xr_enabled.as_deref() {
				Some(XrStatus::Enabled) => aim_pose.single().0,
				_ => *cam_pose.single(),
			};
			let transform = root.single().mul_transform(transform);
			let mut plane_transform = Transform::from_translation(
				transform.translation + *transform.forward(),
			)
			.looking_at(transform.translation, Vec3::Y);
			plane_transform.rotate_local(
				Quat::from_rotation_x(TAU * -1.0 / 4.0) * Quat::from_rotation_y(PI),
			);
			cmds.spawn((
				PbrBundle {
					// The up vec is Y because the code that does the transformation from
					// worldspace to uv space assumes Y up
					mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(1., 1.)),
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
	*last_pressed = pressed;
}
