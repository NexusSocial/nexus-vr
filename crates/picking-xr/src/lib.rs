use bevy::{
	app::{First, Plugin},
	asset::Assets,
	ecs::{
		bundle::Bundle,
		component::Component,
		event::EventWriter,
		query::With,
		schedule::IntoSystemConfigs,
		system::{Query, Res},
	},
	math::{Vec2, Vec3},
	prelude::default,
	render::texture::Image,
	utils::Uuid, transform::components::Transform,
};
use bevy_oxr::{
	resources::XrSession,
	xr_init::xr_only,
	xr_input::{actions::XrActionSets, interactions::XRRayInteractor},
};
use bevy_picking_core::{
	pointer::{self, InputPress, PointerButton, PointerId, PointerLocation},
	PickSet,
};

pub mod raycast_backend;

#[derive(Bundle)]
pub struct XrPointer {
	pub xr_ray_interactor: XRRayInteractor,
	pub pointer_id: PointerId,
	pub pointer_location: PointerLocation,
	pub pick_actions: XrPickActions,
	pub last_action_state: XrPickLastActionState,
	pub last_pointer_hit: XrPickLastPointerTransform,
}

#[derive(Component,Clone, Copy)]
pub struct XrPickLastPointerTransform(Transform);

#[derive(Clone, Copy, Component, Debug)]
pub struct XrActionRef {
	pub action_name: &'static str,
	pub set_name: &'static str,
}

#[derive(Clone, Copy, Component, Debug)]
pub struct XrPickActions {
	/// The Action has to be of type bool
	pub primary_action: XrActionRef,
	/// The Action has to be of type bool
	pub secondary_action: XrActionRef,
	/// The Action has to be of type bool
	pub middle_action: XrActionRef,
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct XrPickLastActionState {
	pub primary_action: bool,
	pub secondary_action: bool,
	pub middle_action: bool,
}

#[derive(Default)]
pub struct XrPickingPlugin;
impl Plugin for XrPickingPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_plugins(raycast_backend::XrRaycastBackend);
		app.add_systems(
			First,
			xr_input_handling.run_if(xr_only()).in_set(PickSet::Input),
		);
	}
}

pub fn xr_input_handling(
	action_sets: Res<XrActionSets>,
	session: Res<XrSession>,
	mut w: EventWriter<InputPress>,
	mut q: Query<
		(&PointerId, &mut XrPickLastActionState, &XrPickActions),
		With<XRRayInteractor>,
	>,
) {
	for (pointer_id, mut last_state, actions) in &mut q {
		let new = action_sets
			.get_action_bool(
				actions.primary_action.set_name,
				actions.primary_action.action_name,
			)
			// Safe
			.unwrap()
			.state(&session, openxr::Path::NULL)
			// unsafe ig
			.unwrap()
			.current_state;

		if new && !last_state.primary_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Down,
				button: PointerButton::Primary,
			})
		}
		if !new && last_state.primary_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Up,
				button: PointerButton::Primary,
			})
		}
		last_state.primary_action = new;

		let new = action_sets
			.get_action_bool(
				actions.secondary_action.set_name,
				actions.secondary_action.action_name,
			)
			// Safe
			.unwrap()
			.state(&session, openxr::Path::NULL)
			// unsafe ig
			.unwrap()
			.current_state;

		if new && !last_state.secondary_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Down,
				button: PointerButton::Secondary,
			})
		}
		if !new && last_state.secondary_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Up,
				button: PointerButton::Secondary,
			})
		}
		last_state.secondary_action = new;
		let new = action_sets
			.get_action_bool(
				actions.middle_action.set_name,
				actions.middle_action.action_name,
			)
			// Safe
			.unwrap()
			.state(&session, openxr::Path::NULL)
			// unsafe ig
			.unwrap()
			.current_state;

		if new && !last_state.middle_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Down,
				button: PointerButton::Middle,
			})
		}
		if !new && last_state.middle_action {
			w.send(InputPress {
				pointer_id: *pointer_id,
				direction: pointer::PressDirection::Up,
				button: PointerButton::Middle,
			})
		}
		last_state.middle_action = new;
	}
}

impl XrPointer {
	pub fn new(assets: &mut Assets<Image>, actions: XrPickActions) -> Self {
		Self {
			last_pointer_hit: XrPickLastPointerTransform(default()),
			xr_ray_interactor: XRRayInteractor,
			pointer_id: PointerId::Custom(Uuid::new_v4()),
			pointer_location: PointerLocation {
				// This is SUPER annoying
				location: Some(pointer::Location {
					target: bevy::render::camera::NormalizedRenderTarget::Image(
						assets.add(Image::default()),
					),
					position: Vec2::ZERO,
				}),
			},
			pick_actions: actions,
			last_action_state: default(),
		}
	}
}
