use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::utils::petgraph::visit::Walker;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_egui::egui::{Key, Ui};
use bevy_egui::systems::{bevy_to_egui_key, bevy_to_egui_physical_key};
use crate::get_egui_keys::{first_row, fn_row, number_row, second_row};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(EguiPlugin)
		// Systems that create Egui widgets should be run during the `CoreSet::Update` set,
		// or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
		.add_systems(Update, (ui_example_system, print_keys).chain())
		.run();
}

pub struct ModifierState {
	caps_lock: bool,
}

impl Default for ModifierState {
	fn default() -> Self {
		Self {
			caps_lock: false,
		}
	}
}

fn ui_example_system(window: Query<Entity, With<PrimaryWindow>>, mut modifier_state: Local<ModifierState>, mut just_pressed: Local<Option<KeyCode>>, mut contexts: EguiContexts, mut keys: ResMut<ButtonInput<KeyCode>>, mut event_writer: EventWriter<KeyboardInput>) {

	let window = window.get_single().unwrap();

	egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
		let mut key_pressed = None;
		ui.horizontal(|ui| {
			show_row(ui, fn_row(), &mut key_pressed);
		});
		ui.horizontal(|ui| {
			show_row(ui, number_row(), &mut key_pressed)
		});
		ui.horizontal(|ui| {
			show_row(ui, first_row(), &mut key_pressed);
		});
		ui.horizontal(|ui| {
			if ui.checkbox(&mut modifier_state.caps_lock, "Caps Lock").clicked() {
				keys.press(KeyCode::CapsLock);
			} else {
				keys.release(KeyCode::CapsLock);
			}
			show_row(ui, second_row(), &mut key_pressed);
		});
		if let Some(previously_pressed) = just_pressed.as_ref() {
			event_writer.send(KeyboardInput {
				key_code: *previously_pressed,
				logical_key: bevy::input::keyboard::Key::Character(bevy_to_egui_physical_key(previously_pressed).unwrap().symbol_or_name().parse().unwrap()),
				state: ButtonState::Released,
				window,
			});
			//keys.release(*previously_pressed);
		}
		if let Some(key) = key_pressed {
			event_writer.send(KeyboardInput {
				key_code: key,
				logical_key: bevy::input::keyboard::Key::Character(bevy_to_egui_physical_key(&key).unwrap().symbol_or_name().parse().unwrap()),
				state: ButtonState::Pressed,
				window,
			});
			//keys.press(key);
			just_pressed.replace(key);
		}
		ui.input(|input| {
			for key in &input.keys_down {
				println!("{}", key.symbol_or_name());
			}
		});
	});
}

fn print_keys(mut buffer: Local<String>, keys: Res<ButtonInput<KeyCode>>, mut contexts: EguiContexts) {
	egui::Window::new("Hello2").show(contexts.ctx_mut(), |ui| {
		/*for key in keys.get_pressed() {
            if key.eq(&KeyCode::Backspace) {
                buffer.pop();
            } else {
                buffer.push_str(format!("{:?}", key).as_str());
            }
        }*/
		let mut_string: &mut String = &mut buffer;
		ui.text_edit_singleline(mut_string);
	});
}

fn show_row(ui: &mut Ui, row: Vec<Key>, key_code: &mut Option<KeyCode>){
	for key in row {
		if let Some(key) = print_key(ui, key) {
			key_code.replace(key);
		}
	}
}

fn print_key(ui: &mut Ui, key: bevy_egui::egui::Key) -> Option<KeyCode> {
	match ui.button(key.symbol_or_name()).clicked() {
		true => Some(convert_egui_key(key)),
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
		_ => panic!("Unhandled key"),
	}


}

mod get_egui_keys {
	use bevy_egui::egui::Key;
	use bevy_egui::egui::Key::*;

	pub fn fn_row() -> Vec<Key> {
		vec![Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12]
	}

	pub fn number_row() -> Vec<Key> {
		vec![Backtick, Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0, Minus, Equals, Backspace]
	}

	pub fn first_row() -> Vec<Key> {
		vec![Tab, Q, W, E, R, T, Y, U, I, O, P, CloseBracket, OpenBracket, Backslash]
	}

	pub fn second_row() -> Vec<Key> {
		vec![A, S, D, F, G, H, J, K, L, Semicolon, Enter]
	}
}