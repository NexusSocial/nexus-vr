//! Plugins for debugging

use bevy::{
	app::{App, Plugin},
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use crate::AppExt;

#[derive(Debug, Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app.add_if_not_added(LogDiagnosticsPlugin::default())
			.add_if_not_added(FrameTimeDiagnosticsPlugin)
			.add_if_not_added(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
			.add_if_not_added(RapierDebugRenderPlugin::default());
	}
}
