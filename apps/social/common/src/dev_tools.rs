//! Plugins related to developer tooling.

use bevy::{
	app::PluginGroupBuilder,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	ecs::component::Component,
	prelude::{App, Local, Plugin, PluginGroup, PostUpdate, With, World},
	reflect::Reflect,
	window::PrimaryWindow,
};

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
		window: Query<Entity, (With<PrimaryWindow>, Without<InspectorUiTarget>)>,
	) {
		if let Ok(window) = window.get_single() {
			cmds.entity(window).insert(InspectorUiTarget);
		}
	}
}

use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;

#[derive(Clone, Copy, Component, Reflect, Default)]
pub struct InspectorUiTarget;

#[derive(Default)]
pub struct InspectorUiPlugin;

impl Plugin for InspectorUiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, inspector_ui);
	}
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
	let mut query = world.query_filtered::<&mut EguiContext, With<InspectorUiTarget>>();

	let ctxs = query.iter(world).len();
	for nth_ctx in 0..ctxs {
		let mut egui_context = query
			.iter_mut(world)
			.nth(nth_ctx)
			.expect("should exist")
			.clone();
		draw_inspector_ui(&mut egui_context, world, &mut selected_entities)
	}
}
fn draw_inspector_ui(
	egui_context: &mut EguiContext,
	world: &mut World,
	selected_entities: &mut SelectedEntities,
) {
	egui::SidePanel::left("hierarchy")
		.default_width(200.0)
		.show(egui_context.get_mut(), |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				ui.heading("Hierarchy");

				bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
					world,
					ui,
					selected_entities,
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
