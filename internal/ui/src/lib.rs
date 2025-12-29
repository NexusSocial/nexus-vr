//! The UI for Nexus

pub mod views;

use bevy::log::warn;

#[derive(Debug, Default)]
pub struct NexusUiPlugin;

impl bevy::app::Plugin for NexusUiPlugin {
	fn build(&self, app: &mut bevy::app::App) {
		if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
			// TODO: Check if we should panic instead, or if it can work without
			// bevy_egui
			warn!(
				"bevy_egui plugin is not enabled, so the NexusUiPlugin might be inert"
			);
		}
		crate::views::add_systems(app);
	}
}
