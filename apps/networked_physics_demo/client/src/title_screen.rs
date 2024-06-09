//! Plugins for title screen

use bevy::{
	app::{Plugin, Update},
	ecs::{
		event::{Event, EventWriter},
		schedule::{
			common_conditions::in_state, IntoSystemConfigs as _, NextState, OnEnter,
		},
		system::{Commands, Res, ResMut},
	},
	input::{keyboard::KeyCode, ButtonInput},
	reflect::Reflect,
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use crate::{netcode::ConnectToManager, AppExt, GameModeState};

use self::ui::EventWriters;

#[derive(Debug, Default)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_if_not_added(bevy_inspector_egui::bevy_egui::EguiPlugin)
			.add_if_not_added(crate::netcode::NetcodePlugin)
			.register_type::<ui::TitleScreen>()
			.add_systems(OnEnter(GameModeState::TitleScreen), setup)
			.add_systems(
				Update,
				(should_transition, draw_title_screen)
					.run_if(in_state(GameModeState::TitleScreen)),
			);
	}
}

/// Contains ui state.
mod ui {
	use bevy::{ecs::system::Resource, prelude::default};

	use super::*;

	/// [`EventWriter`]s needed by the UI.
	pub(super) struct EventWriters<'a> {
		pub connect_to_manager: EventWriter<'a, ConnectToManager>,
	}

	impl EventWriters<'_> {
		/// Sends the ui event `evt`.
		fn send(&mut self, evt: impl UiEvent) {
			evt.send(self)
		}
	}

	/// Events that can be sent with [`EventWriters`].
	trait UiEvent: Sized + Event {
		/// Sends the event with `evw`
		fn send(self, evw: &mut EventWriters<'_>);
	}

	impl UiEvent for ConnectToManager {
		fn send(self, evw: &mut EventWriters<'_>) {
			evw.connect_to_manager.send(self);
		}
	}

	#[derive(Debug, Default, Resource, Reflect, Eq, PartialEq, derive_more::From)]
	pub enum TitleScreen {
		#[default]
		Initial,
		Create(CreateInstance),
		Join(JoinInstance),
	}

	impl TitleScreen {
		pub fn draw(self, ui: &mut egui::Ui, evw: EventWriters) -> Self {
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
				Self::Create(create) => create.draw(ui, evw),
				Self::Join(join) => join.draw(ui, evw),
			}
		}
	}

	/// User has chosen to create an instance.
	#[derive(Debug, Reflect, Eq, PartialEq)]
	pub enum CreateInstance {
		Initial { manager_url: String },
		WaitingForConnection,
		Connected,
		InstanceCreated,
	}

	impl CreateInstance {
		fn draw(mut self, ui: &mut egui::Ui, mut evw: EventWriters) -> TitleScreen {
			match self {
				CreateInstance::Initial {
					ref mut manager_url,
				} => {
					ui.add(egui::TextEdit::singleline(manager_url).hint_text(
						"Manager Url (leave blank to spin up server locally)",
					));
					let text = if manager_url.is_empty() {
						"Locally Host"
					} else {
						"Remotely Host"
					};
					if ui.button(text).clicked() {
						evw.send(ConnectToManager {
							manager_url: manager_url.to_owned(),
						});
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
	#[derive(Debug, Reflect, Eq, PartialEq)]
	pub enum JoinInstance {
		Initial { instance_url: String },
		WaitingForConnection,
		Connected,
	}

	impl JoinInstance {
		fn draw(mut self, ui: &mut egui::Ui, _evw: EventWriters) -> TitleScreen {
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

fn setup(mut commands: Commands) {
	commands.init_resource::<ui::TitleScreen>()
}

fn draw_title_screen(
	mut state: ResMut<ui::TitleScreen>,
	mut contexts: EguiContexts,
	connect_to_manager: EventWriter<ConnectToManager>,
) {
	egui::Window::new("Instances")
		.resizable(false)
		.movable(false)
		.collapsible(false)
		.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
		.show(contexts.ctx_mut(), |ui: &mut egui::Ui| {
			let evw = EventWriters { connect_to_manager };
			// need ownership of state, so replace with the default temporarily
			let stolen = std::mem::take(state.as_mut());
			*state = stolen.draw(ui, evw);
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
