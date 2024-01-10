//! Plugins related to developer tooling.

use bevy::{
	app::PluginGroupBuilder,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	prelude::{App, Plugin, PluginGroup, With},
	window::PrimaryWindow,
};

use crate::inspector::InspectorUiPlugin;
pub use crate::inspector::InspectorUiRenderTarget;

#[cfg(not(target_os = "android"))]
mod not_android {
	use bevy::{
		app::Startup,
		ecs::{
			entity::Entity,
			query::Without,
			system::{Commands, Query},
		},
	};

	use crate::inspector::InspectorUiRenderTarget;

	use super::*;
	#[derive(Default)]
	pub struct PcWindowInspectorPlugin;
	impl Plugin for PcWindowInspectorPlugin {
		fn build(&self, app: &mut App) {
			app.add_systems(Startup, attach_inspector_to_primary_windows);
		}
	}

	fn attach_inspector_to_primary_windows(
		mut cmds: Commands,
		window: Query<Entity, (With<PrimaryWindow>, Without<InspectorUiRenderTarget>)>,
	) {
		if let Ok(window) = window.get_single() {
			cmds.entity(window).insert(InspectorUiRenderTarget);
		}
	}
}

#[derive(Default)]
pub struct DevToolsPlugins;

impl PluginGroup for DevToolsPlugins {
	fn build(self) -> PluginGroupBuilder {
		let builder = PluginGroupBuilder::start::<Self>()
			.add(LogDiagnosticsPlugin::default())
			.disable::<LogDiagnosticsPlugin>()
			.add(FrameTimeDiagnosticsPlugin)
			.add(bevy_inspector_egui::DefaultInspectorConfigPlugin)
			.add(InspectorUiPlugin);
		#[cfg(not(target_os = "android"))]
		let builder = builder.add(self::not_android::PcWindowInspectorPlugin);
		builder
	}
}
