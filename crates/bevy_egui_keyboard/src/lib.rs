use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::get_egui_keys::{first_row, fn_row, number_row, second_row, third_row};
use bevy_egui::egui::{Key, Ui, WidgetText};
use bevy_egui::systems::bevy_to_egui_physical_key;

#[derive(Clone, Copy)]
pub struct ModifierState {
	caps_lock: bool,
}

impl Default for ModifierState {
	fn default() -> Self {
		Self { caps_lock: false }
	}
}
/*

pub fn bevy_to_egui_physical_key(key: &KeyCode) -> Option<egui::Key> {
	let key = match key {
		KeyCode::ArrowDown => egui::Key::ArrowDown,
		KeyCode::ArrowLeft => egui::Key::ArrowLeft,
		KeyCode::ArrowRight => egui::Key::ArrowRight,
		KeyCode::ArrowUp => egui::Key::ArrowUp,

		KeyCode::Escape => egui::Key::Escape,
		KeyCode::Tab => egui::Key::Tab,
		KeyCode::Backspace => egui::Key::Backspace,
		KeyCode::Enter | KeyCode::NumpadEnter => egui::Key::Enter,

		KeyCode::Insert => egui::Key::Insert,
		KeyCode::Delete => egui::Key::Delete,
		KeyCode::Home => egui::Key::Home,
		KeyCode::End => egui::Key::End,
		KeyCode::PageUp => egui::Key::PageUp,
		KeyCode::PageDown => egui::Key::PageDown,

		// Punctuation
		KeyCode::Space => egui::Key::Space,
/*		KeyCode::Comma => egui::Key::Comma,
		KeyCode::Period => egui::Key::Period,
		// KeyCode::Colon => egui::Key::Colon, // NOTE: there is no physical colon key on an american keyboard
		KeyCode::Semicolon => egui::Key::Semicolon,
		KeyCode::Backslash => egui::Key::Backslash,
		KeyCode::Slash | KeyCode::NumpadDivide => egui::Key::Slash,
		KeyCode::BracketLeft => egui::Key::OpenBracket,
		KeyCode::BracketRight => egui::Key::CloseBracket,
		KeyCode::Backquote => egui::Key::Backtick,

		KeyCode::Cut => egui::Key::Cut,
		KeyCode::Copy => egui::Key::Copy,
		KeyCode::Paste => egui::Key::Paste,*/
		KeyCode::Minus | KeyCode::NumpadSubtract => egui::Key::Minus,
/*		KeyCode::NumpadAdd => egui::Key::Plus,
		KeyCode::Equal => egui::Key::Equals,*/

		KeyCode::Digit0 | KeyCode::Numpad0 => egui::Key::Num0,
		KeyCode::Digit1 | KeyCode::Numpad1 => egui::Key::Num1,
		KeyCode::Digit2 | KeyCode::Numpad2 => egui::Key::Num2,
		KeyCode::Digit3 | KeyCode::Numpad3 => egui::Key::Num3,
		KeyCode::Digit4 | KeyCode::Numpad4 => egui::Key::Num4,
		KeyCode::Digit5 | KeyCode::Numpad5 => egui::Key::Num5,
		KeyCode::Digit6 | KeyCode::Numpad6 => egui::Key::Num6,
		KeyCode::Digit7 | KeyCode::Numpad7 => egui::Key::Num7,
		KeyCode::Digit8 | KeyCode::Numpad8 => egui::Key::Num8,
		KeyCode::Digit9 | KeyCode::Numpad9 => egui::Key::Num9,

		KeyCode::KeyA => egui::Key::A,
		KeyCode::KeyB => egui::Key::B,
		KeyCode::KeyC => egui::Key::C,
		KeyCode::KeyD => egui::Key::D,
		KeyCode::KeyE => egui::Key::E,
		KeyCode::KeyF => egui::Key::F,
		KeyCode::KeyG => egui::Key::G,
		KeyCode::KeyH => egui::Key::H,
		KeyCode::KeyI => egui::Key::I,
		KeyCode::KeyJ => egui::Key::J,
		KeyCode::KeyK => egui::Key::K,
		KeyCode::KeyL => egui::Key::L,
		KeyCode::KeyM => egui::Key::M,
		KeyCode::KeyN => egui::Key::N,
		KeyCode::KeyO => egui::Key::O,
		KeyCode::KeyP => egui::Key::P,
		KeyCode::KeyQ => egui::Key::Q,
		KeyCode::KeyR => egui::Key::R,
		KeyCode::KeyS => egui::Key::S,
		KeyCode::KeyT => egui::Key::T,
		KeyCode::KeyU => egui::Key::U,
		KeyCode::KeyV => egui::Key::V,
		KeyCode::KeyW => egui::Key::W,
		KeyCode::KeyX => egui::Key::X,
		KeyCode::KeyY => egui::Key::Y,
		KeyCode::KeyZ => egui::Key::Z,

		KeyCode::F1 => egui::Key::F1,
		KeyCode::F2 => egui::Key::F2,
		KeyCode::F3 => egui::Key::F3,
		KeyCode::F4 => egui::Key::F4,
		KeyCode::F5 => egui::Key::F5,
		KeyCode::F6 => egui::Key::F6,
		KeyCode::F7 => egui::Key::F7,
		KeyCode::F8 => egui::Key::F8,
		KeyCode::F9 => egui::Key::F9,
		KeyCode::F10 => egui::Key::F10,
		KeyCode::F11 => egui::Key::F11,
		KeyCode::F12 => egui::Key::F12,
		KeyCode::F13 => egui::Key::F13,
		KeyCode::F14 => egui::Key::F14,
		KeyCode::F15 => egui::Key::F15,
		KeyCode::F16 => egui::Key::F16,
		KeyCode::F17 => egui::Key::F17,
		KeyCode::F18 => egui::Key::F18,
		KeyCode::F19 => egui::Key::F19,
		KeyCode::F20 => egui::Key::F20,
		_ => return None,
	};
	Some(key)
}
*/

#[derive(Clone, Debug)]
pub enum KeyValue {
	Char(String),
	Key(Key),
	CharCode(String, KeyCode),
}

impl From<Key> for KeyValue {
	fn from(value: Key) -> Self {
		Self::Key(value)
	}
}
impl From<String> for KeyValue {
	fn from(value: String) -> Self {
		Self::Char(value)
	}
}
impl From<&str> for KeyValue {
	fn from(value: &str) -> Self {
		Self::Char(value.into())
	}
}
impl KeyValue {
	pub fn symbol_or_name(&self, modifier_state: &ModifierState) -> String {
		match self {
			KeyValue::Char(char) => match modifier_state.caps_lock {
				true => char.to_uppercase(),
				false => char.to_lowercase(),
			},
			KeyValue::Key(key) => key.symbol_or_name().to_string(),
			KeyValue::CharCode(char, _) => match modifier_state.caps_lock {
				true => char.to_uppercase(),
				false => char.to_lowercase(),
			},
		}
	}
}

pub fn draw_keyboard(
	ui: &mut Ui,
	primary_window: Entity,
	previously_pressed: &mut Local<Option<KeyValue>>,
	keyboard_writer: &mut EventWriter<KeyboardInput>,
	char_writer: &mut EventWriter<ReceivedCharacter>,
	modifier_state: &mut ModifierState,
	on_screen_keyboard: &mut OnScreenKeyboard,
) {
	let curr_modifier_state = *modifier_state;
	let mut key_pressed = None;
	ui.horizontal(|ui| {
		show_row(ui, fn_row(), &mut key_pressed, &curr_modifier_state);
	});
	ui.horizontal(|ui| {
		show_row(ui, number_row(), &mut key_pressed, &curr_modifier_state)
	});
	ui.horizontal(|ui| {
		show_row(ui, first_row(), &mut key_pressed, &curr_modifier_state);
	});
	ui.horizontal(|ui| {
		if ui
			.checkbox(&mut modifier_state.caps_lock, "Caps Lock")
			.clicked()
		{
			let button_state = match modifier_state.caps_lock {
				true => ButtonState::Pressed,
				false => ButtonState::Released,
			};
			keyboard_writer.send(KeyboardInput {
				key_code: KeyCode::CapsLock,
				logical_key: bevy::input::keyboard::Key::CapsLock,
				state: button_state,
				window: primary_window,
			});
		}
		show_row(ui, second_row(), &mut key_pressed, &curr_modifier_state);
	});
	ui.horizontal(|ui| {
		show_row(ui, third_row(), &mut key_pressed, &curr_modifier_state);
	});

	on_screen_keyboard.0 = key_pressed.is_some();

	// if let Some(previously_pressed) = previously_pressed.take() {
	// 	keyboard_writer.send(KeyboardInput {
	// 		key_code: previously_pressed,
	// 		logical_key: bevy::input::keyboard::Key::Character(
	// 			bevy_to_egui_physical_key(&previously_pressed)
	// 				.unwrap()
	// 				.symbol_or_name()
	// 				.parse()
	// 				.unwrap(),
	// 		),
	// 		state: ButtonState::Released,
	// 		window: primary_window,
	// 	});
	// }
	if let Some(key) = key_pressed {
		match key.clone() {
			KeyValue::Char(char) => {
				char_writer.send(ReceivedCharacter {
					window: primary_window,
					char: match curr_modifier_state.caps_lock {
						true => char.to_uppercase(),
						false => char.to_lowercase(),
					}
					.into(),
				});
			}
			KeyValue::Key(key) => {
				let key = convert_egui_key(key);
				keyboard_writer.send(KeyboardInput {
					key_code: key,
					logical_key: bevy::input::keyboard::Key::Character(
						bevy_to_egui_physical_key(&key)
							.unwrap()
							.symbol_or_name()
							.parse()
							.unwrap(),
					),
					state: ButtonState::Pressed,
					window: primary_window,
				});
			}
			KeyValue::CharCode(_, _) => todo!(),
		}
		previously_pressed.replace(key);
	}
}

fn show_row(
	ui: &mut Ui,
	row: Vec<KeyValue>,
	key_code: &mut Option<KeyValue>,
	modifier_state: &ModifierState,
) {
	for key in row {
		if let Some(key) = print_key(ui, key, modifier_state) {
			key_code.replace(key);
		}
	}
}

fn print_key(
	ui: &mut Ui,
	key: KeyValue,
	modifier_state: &ModifierState,
) -> Option<KeyValue> {
	let text: WidgetText = key.symbol_or_name(modifier_state).into();
	match ui.button(text.monospace()).clicked() {
		true => Some(key),
		false => None,
	}
}

fn convert_egui_key(key: bevy_egui::egui::Key) -> KeyCode {
	match key {
		Key::Escape => KeyCode::Escape,
		Key::Tab => KeyCode::Tab,
		Key::Backspace => KeyCode::Backspace,
		Key::Enter => KeyCode::Enter,
		Key::Space => KeyCode::Space,
		Key::Delete => KeyCode::Delete,
		Key::Colon => todo!(),
		Key::Comma => KeyCode::Comma,
		Key::Backslash => KeyCode::Backslash,
		Key::Slash => KeyCode::Slash,
		Key::Pipe => todo!(),
		Key::Questionmark => todo!(),
		Key::OpenBracket => KeyCode::BracketLeft,
		Key::CloseBracket => KeyCode::BracketRight,
		Key::Backtick => KeyCode::Backquote,
		Key::Minus => KeyCode::Minus,
		Key::Period => KeyCode::Period,
		Key::Plus => todo!(),
		Key::Equals => KeyCode::Equal,
		Key::Semicolon => KeyCode::Semicolon,
		Key::Num0 => KeyCode::Digit0,
		Key::Num1 => KeyCode::Digit1,
		Key::Num2 => KeyCode::Digit2,
		Key::Num3 => KeyCode::Digit3,
		Key::Num4 => KeyCode::Digit4,
		Key::Num5 => KeyCode::Digit5,
		Key::Num6 => KeyCode::Digit6,
		Key::Num7 => KeyCode::Digit7,
		Key::Num8 => KeyCode::Digit8,
		Key::Num9 => KeyCode::Digit9,
		Key::A => KeyCode::KeyA,
		Key::B => KeyCode::KeyB,
		Key::C => KeyCode::KeyC,
		Key::D => KeyCode::KeyD,
		Key::E => KeyCode::KeyE,
		Key::F => KeyCode::KeyF,
		Key::G => KeyCode::KeyG,
		Key::H => KeyCode::KeyH,
		Key::I => KeyCode::KeyI,
		Key::J => KeyCode::KeyJ,
		Key::K => KeyCode::KeyK,
		Key::L => KeyCode::KeyL,
		Key::M => KeyCode::KeyM,
		Key::N => KeyCode::KeyN,
		Key::O => KeyCode::KeyO,
		Key::P => KeyCode::KeyP,
		Key::Q => KeyCode::KeyQ,
		Key::R => KeyCode::KeyR,
		Key::S => KeyCode::KeyS,
		Key::T => KeyCode::KeyT,
		Key::U => KeyCode::KeyU,
		Key::V => KeyCode::KeyV,
		Key::W => KeyCode::KeyW,
		Key::X => KeyCode::KeyX,
		Key::Y => KeyCode::KeyY,
		Key::Z => KeyCode::KeyZ,
		Key::F1 => KeyCode::F1,
		Key::F2 => KeyCode::F2,
		Key::F3 => KeyCode::F3,
		Key::F4 => KeyCode::F4,
		Key::F5 => KeyCode::F5,
		Key::F6 => KeyCode::F6,
		Key::F7 => KeyCode::F7,
		Key::F8 => KeyCode::F8,
		Key::F9 => KeyCode::F9,
		Key::F10 => KeyCode::F10,
		Key::F11 => KeyCode::F11,
		Key::F12 => KeyCode::F12,
		_ => panic!("Unhandled key"),
	}
}

pub struct EguiKeyboard;

impl Plugin for EguiKeyboard {
	fn build(&self, app: &mut App) {
		app.insert_resource(OnScreenKeyboard(false));
	}
}
#[derive(Resource)]
pub struct OnScreenKeyboard(pub bool);

mod get_egui_keys {
	// use bevy_egui::egui::Key;
	use bevy_egui::egui::Key::*;

	use crate::KeyValue;
	use crate::KeyValue::Char;
	use crate::KeyValue::Key;

	pub fn fn_row() -> Vec<KeyValue> {
		vec![
			Key(Escape),
			Key(F1),
			Key(F2),
			Key(F3),
			Key(F4),
			Key(F5),
			Key(F6),
			Key(F7),
			Key(F8),
			Key(F9),
			Key(F10),
			Key(F11),
			Key(F12),
		]
	}

	pub fn number_row() -> Vec<KeyValue> {
		vec![
			Key(Backtick),
			"0".into(),
			"1".into(),
			"2".into(),
			"3".into(),
			"4".into(),
			"5".into(),
			"6".into(),
			"7".into(),
			"8".into(),
			"9".into(),
			"0".into(),
			"-".into(),
			"=".into(),
			Key(Backspace),
		]
	}

	pub fn first_row() -> Vec<KeyValue> {
		vec![
			Key(Tab),
			"q".into(),
			"w".into(),
			"e".into(),
			"r".into(),
			"t".into(),
			"y".into(),
			"u".into(),
			"i".into(),
			"o".into(),
			"p".into(),
			")".into(),
			"(".into(),
			"\\".into(),
		]
	}

	pub fn second_row() -> Vec<KeyValue> {
		vec![
			"a".into(),
			"s".into(),
			"d".into(),
			"f".into(),
			"g".into(),
			"h".into(),
			"j".into(),
			"k".into(),
			"l".into(),
			";".into(),
			Key(Enter),
		]
	}

	pub fn third_row() -> Vec<KeyValue> {
		vec![
			"z".into(),
			"x".into(),
			"c".into(),
			"v".into(),
			"b".into(),
			"n".into(),
			"m".into(),
			",".into(),
			".".into(),
			"/".into(),
		]
	}
}
