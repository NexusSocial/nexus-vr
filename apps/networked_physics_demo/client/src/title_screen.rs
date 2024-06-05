//! Plugins for title screen

use bevy::{
	app::{Plugin, Update},
	ecs::{
		schedule::{common_conditions::in_state, IntoSystemConfigs as _, NextState},
		system::{Res, ResMut},
	},
	input::{keyboard::KeyCode, ButtonInput},
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use crate::{AppExt, GameModeState};

#[derive(Debug, Default)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_if_not_added(bevy_inspector_egui::bevy_egui::EguiPlugin)
			// TODO: init this every time we enter the title screen state, instead
			// of just once at plugin build.
			.init_resource::<ui::TitleScreen>()
			.add_systems(
				Update,
				(should_transition, draw_title_screen)
					.run_if(in_state(GameModeState::TitleScreen)),
			);
	}
}

/// Contains ui state.
mod ui {
	use super::egui;
	use bevy::{ecs::system::Resource, reflect::Reflect, utils::default};

	#[derive(Default, Resource, Reflect, Eq, PartialEq, derive_more::From)]
	pub enum TitleScreen {
		#[default]
		Initial,
		Create(CreateInstance),
		Join(JoinInstance),
	}

	impl TitleScreen {
		pub fn draw(self, ui: &mut egui::Ui) -> Self {
			match self {
				Self::Initial => {
					if ui.button("Create Instance").clicked() {
						return Self::Create(default());
					}
					if ui.button("Join Instance").clicked() {
						return Self::Join(default());
					}
					self
				}
				Self::Create(create) => create.draw(ui),
				Self::Join(join) => join.draw(ui),
			}
		}
	}

	/// User has chosen to create an instance.
	#[derive(Reflect, Eq, PartialEq)]
	pub enum CreateInstance {
		Initial { manager_url: String },
		WaitingForConnection,
		Connected,
		InstanceCreated,
	}

	impl CreateInstance {
		fn draw(mut self, ui: &mut egui::Ui) -> TitleScreen {
			match self {
				CreateInstance::Initial {
					ref mut manager_url,
				} => {
					ui.add(
						egui::TextEdit::singleline(manager_url)
							.hint_text("Manager Url"),
					);
					if ui.button("Submit").clicked() {
						return CreateInstance::WaitingForConnection.into();
					}
					if ui.button("Back").clicked() {
						return default();
					}
					self.into()
				}
				CreateInstance::WaitingForConnection => {
					ui.spinner();
					self.into()
				}
				CreateInstance::Connected => todo!(),
				CreateInstance::InstanceCreated => todo!(),
			}
		}
	}

	impl Default for CreateInstance {
		fn default() -> Self {
			Self::Initial {
				manager_url: default(),
			}
		}
	}

	/// User has chosen to join an instance.
	#[derive(Reflect, Eq, PartialEq)]
	pub enum JoinInstance {
		Initial { instance_url: String },
		WaitingForConnection,
		Connected,
	}

	impl JoinInstance {
		fn draw(mut self, ui: &mut egui::Ui) -> TitleScreen {
			match self {
				JoinInstance::Initial {
					ref mut instance_url,
				} => {
					ui.add(
						egui::TextEdit::singleline(instance_url)
							.hint_text("Instance Url"),
					);
					if ui.button("Submit").clicked() {
						return JoinInstance::WaitingForConnection.into();
					}
					if ui.button("Back").clicked() {
						return default();
					}
					self.into()
				}
				JoinInstance::WaitingForConnection => {
					ui.spinner();
					self.into()
				}
				JoinInstance::Connected => todo!(),
			}
		}
	}

	impl Default for JoinInstance {
		fn default() -> Self {
			Self::Initial {
				instance_url: default(),
			}
		}
	}
}

fn draw_title_screen(mut state: ResMut<ui::TitleScreen>, mut contexts: EguiContexts) {
	egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
		// need ownership of state, so replace with the default temporarily
		let stolen = std::mem::take(state.as_mut());
		*state = stolen.draw(ui);
	});
}

/// Emits a state change under certain conditions.
fn should_transition(
	kb: Res<ButtonInput<KeyCode>>,
	mut next_state: ResMut<NextState<GameModeState>>,
) {
	if kb.just_pressed(KeyCode::Space) {
		next_state.set(GameModeState::InMinecraft)
	}
}
