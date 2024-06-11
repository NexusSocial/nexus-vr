#![allow(clippy::type_complexity)]

use bevy::{
	app::{App, Plugin, Update},
	ecs::{
		event::EventReader,
		schedule::{StateTransitionEvent, States},
	},
	log::{debug, info, trace},
	prelude::bevy_main,
	reflect::Reflect,
	DefaultPlugins,
};

mod debug;
mod game;
mod netcode;
mod rng;
mod title_screen;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[bevy_main]
pub fn main() {
	color_eyre::install().unwrap();
	info!("Running {}", APP_NAME);
	debug!("debug");
	trace!("trace");

	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Update, log_transitions)
		.init_state::<GameModeState>()
		.add_plugins(crate::title_screen::TitleScreenPlugin)
		.add_plugins(crate::game::GamePlugin)
		.add_plugins(crate::debug::DebugPlugin)
		.run();
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
enum GameModeState {
	#[default]
	TitleScreen,
	InMinecraft,
}

/// Print when an [`GameModeState`] transition happens.
fn log_transitions(mut transitions: EventReader<StateTransitionEvent<GameModeState>>) {
	for transition in transitions.read() {
		info!(
			"transition: {:?} => {:?}",
			transition.before, transition.after
		);
	}
}

trait AppExt {
	fn add_if_not_added<P: Plugin>(&mut self, plugin: P) -> &mut Self;
}

impl AppExt for bevy::app::App {
	fn add_if_not_added<P: Plugin>(&mut self, plugin: P) -> &mut Self {
		if !self.is_plugin_added::<P>() {
			self.add_plugins(plugin);
		}
		self
	}
}
