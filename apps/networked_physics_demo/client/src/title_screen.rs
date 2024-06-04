//! Plugins for title screen

use bevy::{
	app::{Plugin, Update},
	ecs::{
		schedule::{
			common_conditions::in_state, IntoSystemConfigs as _, NextState, OnEnter,
		},
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
			.add_systems(OnEnter(GameModeState::TitleScreen), setup)
			.add_systems(
				Update,
				should_transition.run_if(in_state(GameModeState::TitleScreen)),
			);
	}
}

/// Run when we enter [`GameModeState::TitleScreen`].
fn setup(mut contexts: EguiContexts) {
	egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
		ui.label("world");
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
