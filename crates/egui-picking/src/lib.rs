use bevy::{ecs::schedule::Condition, prelude::*, utils::HashMap};
use bevy_egui::{egui::PointerButton, EguiInput, EguiRenderToTexture};
use bevy_mod_picking::{
	events::{Down, Move, Out, Pointer, Up},
	focus::PickingInteraction,
	picking_core::Pickable,
	pointer::PointerId,
	prelude::{ListenerInput, On},
};

#[derive(Clone, Copy, Component, Debug)]
pub struct WorldUI {
	pub size_x: f32,
	pub size_y: f32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct UIPointerDown {
	pub target: Entity,
	pub position: Option<Vec3>,
	pub button: PointerButton,
}
#[derive(Event, Clone, Copy, Debug)]
pub struct UIPointerUp {
	pub target: Entity,
	pub position: Option<Vec3>,
	pub button: PointerButton,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct UIPointerMove {
	pub target: Entity,
	pub position: Option<Vec3>,
	pub normal: Option<Vec3>,
	pub pointer_id: PointerId,
}
#[derive(Event, Clone, Copy, Debug)]
pub struct UIPointerLeave {
	pub target: Entity,
	pub pointer_id: PointerId,
}

impl From<ListenerInput<Pointer<Down>>> for UIPointerDown {
	fn from(event: ListenerInput<Pointer<Down>>) -> Self {
		Self {
			target: event.target,
			position: event.hit.position,
			button: match event.button {
				bevy_mod_picking::pointer::PointerButton::Primary => {
					PointerButton::Primary
				}
				bevy_mod_picking::pointer::PointerButton::Secondary => {
					PointerButton::Secondary
				}
				bevy_mod_picking::pointer::PointerButton::Middle => {
					PointerButton::Middle
				}
			},
		}
	}
}
impl From<ListenerInput<Pointer<Up>>> for UIPointerUp {
	fn from(event: ListenerInput<Pointer<Up>>) -> Self {
		Self {
			target: event.target,
			position: event.hit.position,
			button: match event.button {
				bevy_mod_picking::pointer::PointerButton::Primary => {
					PointerButton::Primary
				}
				bevy_mod_picking::pointer::PointerButton::Secondary => {
					PointerButton::Secondary
				}
				bevy_mod_picking::pointer::PointerButton::Middle => {
					PointerButton::Middle
				}
			},
		}
	}
}

impl From<ListenerInput<Pointer<Move>>> for UIPointerMove {
	fn from(event: ListenerInput<Pointer<Move>>) -> Self {
		Self {
			target: event.target,
			position: event.hit.position,
			normal: event.hit.normal,
			pointer_id: event.pointer_id,
		}
	}
}
impl From<ListenerInput<Pointer<Out>>> for UIPointerLeave {
	fn from(event: ListenerInput<Pointer<Out>>) -> Self {
		Self {
			target: event.target,
			pointer_id: event.pointer_id,
		}
	}
}

#[derive(Bundle)]
pub struct WorldSpaceUI {
	pub on_move: On<Pointer<Move>>,
	pub on_down: On<Pointer<Down>>,
	pub on_up: On<Pointer<Up>>,
	pub on_leave: On<Pointer<Out>>,
	pub render_texture: EguiRenderToTexture,
	pub pickable: Pickable,
	pub interaction: PickingInteraction,
	pub world_ui: WorldUI,
	pub current_pointers: CurrentPointers,
}
impl WorldSpaceUI {
	pub fn new(texture: Handle<Image>, size_x: f32, size_y: f32) -> Self {
		WorldSpaceUI {
			on_move: On::<Pointer<Move>>::send_event::<UIPointerMove>(),
			on_leave: On::<Pointer<Out>>::send_event::<UIPointerLeave>(),
			on_down: On::<Pointer<Down>>::send_event::<UIPointerDown>(),
			on_up: On::<Pointer<Up>>::send_event::<UIPointerUp>(),
			render_texture: EguiRenderToTexture(texture),
			pickable: Pickable::default(),
			interaction: PickingInteraction::default(),
			world_ui: WorldUI { size_x, size_y },
			current_pointers: CurrentPointers::default(),
		}
	}
}

pub struct PickabelEguiPlugin;
impl Plugin for PickabelEguiPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<UIPointerMove>();
		app.add_event::<UIPointerLeave>();
		app.add_event::<UIPointerDown>();
		app.add_event::<UIPointerUp>();
		app.add_systems(
			Update,
			ui_interactions.run_if(
				on_event::<UIPointerMove>()
					.or_else(on_event::<UIPointerLeave>())
					.or_else(on_event::<UIPointerDown>())
					.or_else(on_event::<UIPointerUp>()),
			),
		);
	}
}

#[derive(Component, Default)]
pub struct CurrentPointers {
	pub pointers: HashMap<PointerId, (Vec3, Vec3)>,
}

pub fn ui_interactions(
	mut inputs: Query<(
		&mut EguiInput,
		&WorldUI,
		&GlobalTransform,
		&EguiRenderToTexture,
		&mut CurrentPointers,
	)>,
	mut move_events: EventReader<UIPointerMove>,
	mut down_events: EventReader<UIPointerDown>,
	mut up_events: EventReader<UIPointerUp>,
	mut leave_events: EventReader<UIPointerLeave>,
	textures: Res<Assets<Image>>,
) {
	for UIPointerMove {
		target,
		position,
		pointer_id,
		normal,
	} in move_events.read()
	{
		if let (
			Ok((mut input, ui, transform, texture, mut current_pointers)),
			Some(position),
			Some(normal),
		) = (inputs.get_mut(*target), position, normal)
		{
			current_pointers
				.pointers
				.insert(*pointer_id, (*position, *normal));
			let local_pos = *position - transform.translation();
			let rotated_point = transform
				.to_scale_rotation_translation()
				.1
				.inverse()
				.mul_vec3(local_pos);
			let mut uv = rotated_point.xz() + Vec2::splat(0.5);
			uv.x /= ui.size_x;
			uv.y /= ui.size_y;
			let image = textures.get(texture.0.clone()).unwrap();
			input.events.push(bevy_egui::egui::Event::PointerMoved(
				bevy_egui::egui::Pos2 {
					x: uv.x * image.width() as f32,
					y: uv.y * image.height() as f32,
				},
			));
		}
	}
	for UIPointerDown {
		target,
		position,
		button,
	} in down_events.read()
	{
		if let (
			Ok((mut input, ui, transform, texture, _current_pointers)),
			Some(position),
		) = (inputs.get_mut(*target), position)
		{
			let local_pos = *position - transform.translation();
			let rotated_point = transform
				.to_scale_rotation_translation()
				.1
				.inverse()
				.mul_vec3(local_pos);
			let mut uv = rotated_point.xz() + Vec2::splat(0.5);
			uv.x /= ui.size_x;
			uv.y /= ui.size_y;

			let image = textures.get(texture.0.clone()).unwrap();
			input.events.push(bevy_egui::egui::Event::PointerButton {
				pos: bevy_egui::egui::Pos2 {
					x: uv.x * image.width() as f32,
					y: uv.y * image.height() as f32,
				},
				button: *button,
				pressed: true,
				modifiers: bevy_egui::egui::Modifiers::NONE,
			});
		}
	}
	for UIPointerUp {
		target,
		position,
		button,
	} in up_events.read()
	{
		if let (
			Ok((mut input, ui, transform, texture, _current_pointers)),
			Some(position),
		) = (inputs.get_mut(*target), position)
		{
			let local_pos = *position - transform.translation();
			let rotated_point = transform
				.to_scale_rotation_translation()
				.1
				.inverse()
				.mul_vec3(local_pos);
			let mut uv = rotated_point.xz() + Vec2::splat(0.5);
			uv.x /= ui.size_x;
			uv.y /= ui.size_y;
			let image = textures.get(texture.0.clone()).unwrap();
			input.events.push(bevy_egui::egui::Event::PointerButton {
				pos: bevy_egui::egui::Pos2 {
					x: uv.x * image.width() as f32,
					y: uv.y * image.height() as f32,
				},
				button: *button,
				pressed: false,
				modifiers: bevy_egui::egui::Modifiers::NONE,
			});
		}
	}
	for UIPointerLeave { target, pointer_id } in leave_events.read() {
		if let Ok((mut input, _, _, _, mut current_pointers)) = inputs.get_mut(*target)
		{
			current_pointers.pointers.remove(pointer_id);
			input.events.push(bevy_egui::egui::Event::PointerGone);
		}
	}
}
