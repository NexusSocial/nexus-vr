
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;


use bevy::window::PrimaryWindow;
use bevy::{ecs::schedule::Condition, prelude::*, utils::HashMap};
use bevy_egui::egui::Key;
use bevy_egui::systems::{
	bevy_to_egui_key, bevy_to_egui_physical_key,
};

use bevy_egui::{
	egui, egui::PointerButton, EguiInput, EguiRenderToTexture,
	EguiSet,
};
use bevy_egui_keyboard::OnScreenKeyboard;
use bevy_mod_picking::{
	events::{Down, Move, Out, Pointer, Up},
	focus::PickingInteraction,
	picking_core::Pickable,
	pointer::PointerId,
	prelude::{ListenerInput, On},
};
use std::borrow::Cow;
use std::marker::PhantomData;

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
	pub pointer_id: PointerId,
}
#[derive(Event, Clone, Copy, Debug)]
pub struct UIPointerUp {
	pub target: Entity,
	pub position: Option<Vec3>,
	pub button: PointerButton,
	pub pointer_id: PointerId,
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
			pointer_id: event.pointer_id,
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
			pointer_id: event.pointer_id,
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
	pub current_pointer_interaction: CurrentPointerInteraction,
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
			current_pointer_interaction: CurrentPointerInteraction::default(),
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
		app.add_event::<TextInput>();
		app.add_systems(
			Update,
			ui_interactions.run_if(
				on_event::<UIPointerMove>()
					.or_else(on_event::<UIPointerLeave>())
					.or_else(on_event::<UIPointerDown>())
					.or_else(on_event::<UIPointerUp>()),
			),
		);
		app.add_systems(
			PreUpdate,
			(keyboard_interactions
				.after(EguiSet::ProcessInput)
				.before(EguiSet::BeginFrame),),
		);
	}
}

#[derive(Component, Default)]
pub struct CurrentPointers {
	pub pointers: HashMap<PointerId, (Vec3, Vec3)>,
}
#[derive(Component, Default, DerefMut, Deref)]
pub struct CurrentPointerInteraction {
	pub pointer: Option<PointerId>,
}

#[derive(Event, Clone, Debug)]
pub struct TextInput(Cow<'static, str>);

#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct ModifierKeysState {
	shift: bool,
	ctrl: bool,
	alt: bool,
	win: bool,
}

#[allow(missing_docs)]
#[derive(SystemParam)]
pub struct InputResources<'w, 's> {
	#[cfg(all(
		feature = "manage_clipboard",
		not(target_os = "android"),
		not(all(target_arch = "wasm32", not(web_sys_unstable_apis)))
	))]
	pub egui_clipboard: bevy::ecs::system::ResMut<'w, crate::EguiClipboard>,
	pub modifier_keys_state: Local<'s, ModifierKeysState>,
	#[system_param(ignore)]
	_marker: PhantomData<&'w ()>,
}

pub fn keyboard_interactions(
	mut query: Query<&mut EguiInput, With<WorldUI>>,
	mut keyboard_input: EventReader<KeyboardInput>,
	onscreen_keyboard: Res<OnScreenKeyboard>,
	mut text_inputs: EventReader<TextInput>,
	window_query: Query<&EguiInput, (With<PrimaryWindow>, Without<WorldUI>)>,
) {
	// use std::sync::Once;
	// static START: Once = Once::new();
	// START.call_once(|| {
	// 	// The default for WASM is `false` since the `target_os` is `unknown`.
	// 	//*context_params.is_macos = cfg!(target_os = "macos");
	// });
	let Ok(primary_input) = window_query.get_single() else {
		warn!("Unable to find one Primary Window!");
		return;
	};

	for e in text_inputs.read() {
		for mut egui_input in query.iter_mut() {
			egui_input.events.push(egui::Event::Text(e.0.to_string()));
		}
	}

	let keyboard_input_events = keyboard_input.read().cloned().collect::<Vec<_>>();

	let _modifiers = egui::Modifiers {
		alt: false,
		ctrl: false,
		shift: false,
		mac_cmd: false,
		command: false,
	};

	let events = primary_input.events.iter().filter_map(|e| {
		match e {
			egui::Event::Copy => Some(e.clone()),
			egui::Event::Cut => Some(e.clone()),
			egui::Event::Paste(_) => Some(e.clone()),
			egui::Event::Text(_) => Some(e.clone()),
			egui::Event::Key {
				key: _,
				physical_key: _,
				pressed: _,
				repeat: _,
				modifiers: _,
			} => Some(e.clone()),
			// egui::Event::Scroll(_) => Some(e.clone()),
			// egui::Event::Zoom(_) => Some(e.clone()),
			// egui::Event::Touch {
			// 	device_id:_,
			// 	id:_,
			// 	phase:_,
			// 	pos:_,
			// 	force:_,
			// } => Some(e.clone()),
			// egui::Event::MouseWheel {
			// 	unit:_,
			// 	delta:_,
			// 	modifiers:_,
			// } => Some(e.clone()),
			_ => None,
		}
	});

	let mut is_not_text = None;

	for event in &keyboard_input_events {
		for mut egui_input in query.iter_mut() {
			let (Some(key), _physical_key) = (
				bevy_to_egui_key(&event.logical_key),
				bevy_to_egui_physical_key(&event.key_code),
			) else {
				continue;
			};

			if event.state.eq(&ButtonState::Released) {
				break;
			}
			if !should_represent_as_text(&key) {
				is_not_text.replace(true);
				break;
			}
			egui_input
				.events
				.push(egui::Event::Text(key.symbol_or_name().to_ascii_lowercase()));
		}
	}

	if let Some(is_not_text) = is_not_text {
		if is_not_text {
			for mut egui_input in query.iter_mut() {
				egui_input.events.extend(events.clone());
			}
			return;
		}
	}

	if !onscreen_keyboard.0 {
		return;
	}

	for mut egui_input in query.iter_mut() {
		egui_input.events.extend(events.clone());
	}
}

pub fn should_represent_as_text(key: &Key) -> bool {
	match key {
		Key::Colon
		| Key::Comma
		| Key::Backslash
		| Key::Slash
		| Key::Pipe
		| Key::Questionmark
		| Key::OpenBracket
		| Key::CloseBracket
		| Key::Backtick
		| Key::Minus
		| Key::Period
		| Key::Plus
		| Key::Equals
		| Key::Semicolon
		| Key::Num0
		| Key::Num1
		| Key::Num2
		| Key::Num3
		| Key::Num4
		| Key::Num5
		| Key::Num6
		| Key::Num7
		| Key::Num8
		| Key::Num9
		| Key::A
		| Key::B
		| Key::C
		| Key::D
		| Key::E
		| Key::F
		| Key::G
		| Key::H
		| Key::I
		| Key::J
		| Key::K
		| Key::L
		| Key::M
		| Key::N
		| Key::O
		| Key::P
		| Key::Q
		| Key::R
		| Key::S
		| Key::T
		| Key::U
		| Key::V
		| Key::W
		| Key::X
		| Key::Y
		| Key::Z
		| Key::Space => true,
		_ => false,
	}
}

pub fn ui_interactions(
	mut inputs: Query<(
		&mut EguiInput,
		&WorldUI,
		&GlobalTransform,
		&EguiRenderToTexture,
		&mut CurrentPointers,
		&mut CurrentPointerInteraction,
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
			Ok((
				mut input,
				ui,
				transform,
				texture,
				mut current_pointers,
				current_interaction,
			)),
			Some(position),
			Some(normal),
		) = (inputs.get_mut(*target), position, normal)
		{
			current_pointers
				.pointers
				.insert(*pointer_id, (*position, *normal));
			match **current_interaction {
				Some(pointer) if pointer == *pointer_id => (),
				None => (),
				Some(_) => continue,
			}
			let local_pos = *position - transform.translation();
			// info!("pos: {}", position);
			// info!("local_pos: {}", local_pos);
			let rotated_point = transform
				.to_scale_rotation_translation()
				.1
				.inverse()
				.mul_vec3(local_pos);
			let mut uv = rotated_point.xz()
				+ (Vec2::splat(0.5) * Vec2::new(ui.size_x, ui.size_y));
			// info!("ui_pos: {}", uv);
			uv.x /= ui.size_x;
			uv.y /= ui.size_y;
			// info!("normal_ui_pos: {}", uv);
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
		pointer_id,
	} in down_events.read()
	{
		if let (
			Ok((
				mut input,
				ui,
				transform,
				texture,
				_current_pointers,
				mut current_interaction,
			)),
			Some(position),
		) = (inputs.get_mut(*target), position)
		{
			match **current_interaction {
				Some(_) => continue,
				None => **current_interaction = Some(*pointer_id),
			}
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
			input.events.push(bevy_egui::egui::Event::PointerGone);
			input.events.push(bevy_egui::egui::Event::PointerMoved(
				bevy_egui::egui::Pos2 {
					x: uv.x * image.width() as f32,
					y: uv.y * image.height() as f32,
				},
			));
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
		pointer_id,
	} in up_events.read()
	{
		if let (
			Ok((
				mut input,
				ui,
				transform,
				texture,
				_current_pointers,
				mut current_interaction,
			)),
			Some(position),
		) = (inputs.get_mut(*target), position)
		{
			match **current_interaction {
				Some(pointer) if pointer == *pointer_id => **current_interaction = None,
				None => (),
				Some(_) => continue,
			}
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
		if let Ok((mut input, _, _, _, mut current_pointers, mut current_interaction)) =
			inputs.get_mut(*target)
		{
			current_pointers.pointers.remove(pointer_id);
			match **current_interaction {
				Some(pointer) if pointer == *pointer_id => **current_interaction = None,
				None => (),
				Some(_) => continue,
			}
			input.events.push(bevy_egui::egui::Event::PointerGone);
		}
	}
}
