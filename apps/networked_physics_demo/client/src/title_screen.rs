//! Plugins for title screen

use bevy::{
	app::{Plugin, Update},
	ecs::{
		event::{Event, EventReader, EventWriter},
		schedule::{
			common_conditions::in_state, IntoSystemConfigs as _,
			IntoSystemSetConfigs as _, NextState, OnEnter,
		},
		system::{Commands, Res, ResMut},
	},
	log::{info, trace},
	reflect::Reflect,
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use crate::{
	netcode::{
		ConnectToInstanceRequest, ConnectToInstanceResponse, ConnectToManagerRequest,
		ConnectToManagerResponse, CreateInstanceRequest, CreateInstanceResponse,
	},
	AppExt, GameModeState,
};

use self::ui::EventWriters;

#[derive(Debug, Default)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_if_not_added(bevy_inspector_egui::bevy_egui::EguiPlugin)
			.add_if_not_added(crate::netcode::NetcodePlugin::default())
			.register_type::<ui::TitleScreen>()
			.configure_sets(
				Update,
				UiStateSystems.run_if(in_state(GameModeState::TitleScreen)),
			)
			.add_systems(OnEnter(GameModeState::TitleScreen), setup)
			.add_systems(
				Update,
				(
					(
						handle_connect_to_manager_response,
						handle_create_instance_response,
						handle_connect_to_instance_response,
					)
						.in_set(UiStateSystems),
					(should_transition, draw_ui.after(UiStateSystems))
						.run_if(in_state(GameModeState::TitleScreen)),
				),
			);
	}
}

/// Contains ui state.
mod ui {
	use bevy::{ecs::system::Resource, prelude::default};

	use crate::netcode::{
		ConnectToInstanceRequest, ConnectToManagerRequest, CreateInstanceRequest,
	};

	use super::*;

	/// [`EventWriter`]s needed by the UI.
	pub(super) struct EventWriters<'a> {
		pub connect_to_manager: EventWriter<'a, ConnectToManagerRequest>,
		pub create_instance: EventWriter<'a, CreateInstanceRequest>,
		pub connect_to_instance: EventWriter<'a, ConnectToInstanceRequest>,
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

	impl UiEvent for ConnectToManagerRequest {
		fn send(self, evw: &mut EventWriters<'_>) {
			evw.connect_to_manager.send(self);
		}
	}

	impl UiEvent for CreateInstanceRequest {
		fn send(self, evw: &mut EventWriters<'_>) {
			evw.create_instance.send(self);
		}
	}

	impl UiEvent for ConnectToInstanceRequest {
		fn send(self, evw: &mut EventWriters<'_>) {
			evw.connect_to_instance.send(self);
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
		Initial {
			manager_url: String,
			/// Non-empty will display this as the error.
			error_msg: String,
		},
		WaitingForConnection {
			/// The url given in `Initial`
			manager_url: String,
		},
		Connected,
		InstanceCreated {
			instance_url: String,
		},
	}

	impl CreateInstance {
		fn draw(mut self, ui: &mut egui::Ui, mut evw: EventWriters) -> TitleScreen {
			match self {
				CreateInstance::Initial {
					ref mut manager_url,
					ref mut error_msg,
				} => {
					ui.add(
						egui::TextEdit::singleline(manager_url)
							.hint_text(
								"Manager Url (leave blank to spin up server locally)",
							)
							.text_color_opt(
								(!error_msg.is_empty()).then_some(egui::Color32::RED),
							),
					);
					let text = if manager_url.is_empty() {
						error_msg.clear();
						"Locally Host"
					} else if !error_msg.is_empty() {
						error_msg.as_str()
					} else {
						"Remotely Host"
					};
					if ui.button(text).clicked() {
						match (!manager_url.is_empty())
							.then(|| manager_url.parse())
							.transpose()
						{
							Ok(Some(parsed_manager_url)) => {
								evw.send(ConnectToManagerRequest {
									manager_url: parsed_manager_url,
								});
								return CreateInstance::WaitingForConnection {
									manager_url: std::mem::take(manager_url),
								}
								.into();
							}
							Ok(None) => {
								// Locally host
								evw.send(ConnectToInstanceRequest {
									instance_url: None,
								});
								return JoinInstance::WaitingForConnection.into();
							}
							Err(_parse_err) => {
								error_msg.clear();
								error_msg.push_str("Invalid URL");
							}
						}
					}
					if ui.button("Back").clicked() {
						return default();
					}
					self.into()
				}
				CreateInstance::WaitingForConnection { .. } => {
					ui.spinner();
					self.into()
				}
				CreateInstance::Connected => {
					ui.label("Connected to Manager, creating instance...");
					ui.spinner();
					self.into()
				}
				CreateInstance::InstanceCreated {
					ref mut instance_url,
				} => {
					ui.label("Created instance!");
					ui.label(&*instance_url);
					ui.output_mut(|o| instance_url.clone_into(&mut o.copied_text));
					if ui.button("Join Instance").clicked() {
						let instance_url = instance_url.parse().expect("infallible");
						evw.send(ConnectToInstanceRequest {
							instance_url: Some(instance_url),
						});
						return JoinInstance::WaitingForConnection.into();
					}
					self.into()
				}
			}
		}
	}

	impl Default for CreateInstance {
		fn default() -> Self {
			Self::Initial {
				manager_url: default(),
				error_msg: String::new(),
			}
		}
	}

	/// User has chosen to join an instance.
	#[derive(Debug, Reflect, Eq, PartialEq)]
	pub enum JoinInstance {
		Initial {
			instance_url: String,
			error_msg: String,
		},
		WaitingForConnection,
		Connected,
	}

	impl JoinInstance {
		fn draw(mut self, ui: &mut egui::Ui, mut evw: EventWriters) -> TitleScreen {
			match self {
				JoinInstance::Initial {
					ref mut instance_url,
					ref mut error_msg,
				} => {
					ui.add(
						egui::TextEdit::singleline(instance_url)
							.hint_text("Instance Url")
							.text_color_opt(
								(!error_msg.is_empty()).then_some(egui::Color32::RED),
							),
					);
					if instance_url.is_empty() {
						error_msg.clear();
					}
					let text = if error_msg.is_empty() {
						"Connect"
					} else {
						error_msg.as_str()
					};
					if ui
						.add_enabled(!instance_url.is_empty(), egui::Button::new(text))
						.clicked()
					{
						match instance_url.parse() {
							Ok(instance_url) => {
								evw.send(ConnectToInstanceRequest {
									instance_url: Some(instance_url),
								});
								return JoinInstance::WaitingForConnection.into();
							}
							Err(_parse_err) => {
								error_msg.clear();
								error_msg.push_str("Invalid URL");
							}
						}
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
				JoinInstance::Connected => {
					ui.label("Connected!");
					self.into()
				}
			}
		}
	}

	impl Default for JoinInstance {
		fn default() -> Self {
			Self::Initial {
				instance_url: default(),
				error_msg: String::new(),
			}
		}
	}
}

fn setup(mut commands: Commands) {
	commands.init_resource::<ui::TitleScreen>()
}

/// Systems related to transition of UI state
#[derive(bevy::prelude::SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
struct UiStateSystems;

fn handle_connect_to_manager_response(
	mut ui_state: ResMut<ui::TitleScreen>,
	mut connect_to_manager_response: EventReader<ConnectToManagerResponse>,
	mut create_instance_request: EventWriter<CreateInstanceRequest>,
) {
	let ui::TitleScreen::Create(ref mut create_state) = *ui_state else {
		return;
	};
	for response in connect_to_manager_response.read() {
		trace!("handling ConnectToManagerResponse");
		let Err(ref _response_err) = response.0 else {
			*create_state = ui::CreateInstance::Connected;
			create_instance_request.send(CreateInstanceRequest);
			continue;
		};
		if let ui::CreateInstance::WaitingForConnection {
			ref mut manager_url,
		} = *create_state
		{
			*create_state = ui::CreateInstance::Initial {
				error_msg: "Could not connect to server".to_owned(),
				manager_url: std::mem::take(manager_url),
			}
		}
	}
}

fn handle_create_instance_response(
	manager: Option<Res<crate::netcode::NetcodeManager>>,
	mut ui_state: ResMut<ui::TitleScreen>,
	mut create_instance_response: EventReader<CreateInstanceResponse>,
) {
	let ui::TitleScreen::Create(ref mut create_state) = *ui_state else {
		return;
	};
	for response in create_instance_response.read() {
		trace!("handling CreateInstanceResponse");
		match &response.0 {
			Ok(url) => {
				*create_state = ui::CreateInstance::InstanceCreated {
					instance_url: url.to_string(),
				}
			}
			Err(_err) => {
				*create_state = ui::CreateInstance::Initial {
					error_msg: "error while creating instance".to_owned(),
					manager_url: manager
						.as_ref()
						.map(|m| m.url().to_string())
						.unwrap_or_default(),
				}
			}
		}
	}
}

fn handle_connect_to_instance_response(
	mut ui_state: ResMut<ui::TitleScreen>,
	mut connect_to_instance_response: EventReader<ConnectToInstanceResponse>,
) {
	let ui::TitleScreen::Join(ref mut join_state) = *ui_state else {
		return;
	};
	for response in connect_to_instance_response.read() {
		trace!("handling ConnectToInstanceResponse");
		match &response.result {
			Err(_err) => {
				*join_state = ui::JoinInstance::Initial {
					error_msg: "Failed to connect to instance".to_owned(),
					instance_url: response
						.instance_url
						.as_ref()
						.map(|url| url.to_string())
						.unwrap_or_default(),
				}
			}
			Ok(_) => *join_state = ui::JoinInstance::Connected,
		}
		//
	}
}

fn draw_ui(
	mut state: ResMut<ui::TitleScreen>,
	mut contexts: EguiContexts,
	connect_to_manager: EventWriter<ConnectToManagerRequest>,
	create_instance: EventWriter<CreateInstanceRequest>,
	connect_to_instance: EventWriter<ConnectToInstanceRequest>,
) {
	egui::Window::new("Instances")
		.resizable(false)
		.movable(false)
		.collapsible(false)
		.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
		.show(contexts.ctx_mut(), |ui: &mut egui::Ui| {
			let evw = EventWriters {
				connect_to_manager,
				create_instance,
				connect_to_instance,
			};
			// need ownership of state, so replace with the default temporarily
			let stolen = std::mem::take(state.as_mut());
			*state = stolen.draw(ui, evw);
		});
}

/// Emits a state change under certain conditions.
fn should_transition(
	mut connect_to_instance: EventReader<ConnectToInstanceResponse>,
	mut next_state: ResMut<NextState<GameModeState>>,
) {
	if connect_to_instance
		.read()
		.any(|response| response.result.is_ok())
	{
		info!("Connected to instance, transitioning...");
		next_state.set(GameModeState::InMinecraft)
	}
}
