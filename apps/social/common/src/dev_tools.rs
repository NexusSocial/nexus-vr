//! Plugins related to developer tooling.

use bevy::{
	app::PluginGroupBuilder,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	prelude::{App, Local, Plugin, PluginGroup, PostUpdate, With, World},
	window::PrimaryWindow,
};

#[cfg(not(target_os = "android"))]
mod not_android {
	use super::*;

	use bevy_egui::EguiContext;
	use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;

	#[derive(Default)]
	pub struct InspectorUiPlugin;

	impl Plugin for InspectorUiPlugin {
		fn build(&self, app: &mut App) {
			app.add_systems(PostUpdate, inspector_ui);
		}
	}

	fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
		let mut egui_context = world
			.query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
			.single(world)
			.clone();

		egui::SidePanel::left("hierarchy")
			.default_width(200.0)
			.show(egui_context.get_mut(), |ui| {
				egui::ScrollArea::vertical().show(ui, |ui| {
					ui.heading("Hierarchy");

					bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
						world,
						ui,
						&mut selected_entities,
					);

					ui.label("Press escape to toggle UI");
					ui.allocate_space(ui.available_size());
				});
			});

		egui::SidePanel::right("inspector")
			.default_width(250.0)
			.show(egui_context.get_mut(), |ui| {
				egui::ScrollArea::vertical().show(ui, |ui| {
					ui.heading("Inspector");

					match selected_entities.as_slice() {
						&[entity] => {
							bevy_inspector_egui::bevy_inspector::ui_for_entity(
								world, entity, ui,
							);
						}
						entities => {
							bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
							world, entities, ui,
						);
						}
					}

					ui.allocate_space(ui.available_size());
				});
			});
	}
}

#[derive(Default)]
pub struct DevToolsPlugins;

impl PluginGroup for DevToolsPlugins {
	fn build(self) -> PluginGroupBuilder {
		let builder = PluginGroupBuilder::start::<Self>()
			.add(LogDiagnosticsPlugin::default())
			.disable::<LogDiagnosticsPlugin>()
			.add(FrameTimeDiagnosticsPlugin);
		#[cfg(not(target_os = "android"))]
		let builder = builder
			.add(bevy_inspector_egui::DefaultInspectorConfigPlugin)
			// I am sorry ~Schmarni
			// .add(bevy_egui::EguiPlugin)
			.add(self::not_android::InspectorUiPlugin);
		builder
	}
}
