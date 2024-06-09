use bevy::{
	app::{Plugin, Update},
	ecs::{
		event::{Event, EventReader},
		schedule::NextState,
		system::ResMut,
	},
	reflect::Reflect,
};

use crate::GameModeState;

#[derive(Debug)]
pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<ConnectToManager>()
			.add_event::<ConnectToManager>()
			.add_systems(Update, on_connect_to_manager_evt);
	}
}

/// Other plugins create this when they want to connect to a manager.
#[derive(Debug, Reflect, Event, Eq, PartialEq)]
pub struct ConnectToManager {
	/// The URL of the manager to connect to
	pub manager_url: String,
}

fn on_connect_to_manager_evt(
	mut connect_to_manager: EventReader<ConnectToManager>,
	mut next_state: ResMut<NextState<GameModeState>>,
) {
	for ConnectToManager { manager_url: _ } in connect_to_manager.read() {
		// TODO: Actually connect to the manager instead of faking it
		next_state.set(GameModeState::InMinecraft);
	}
}
