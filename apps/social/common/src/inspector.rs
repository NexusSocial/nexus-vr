/// This module is Responsible for rendering the Entity Inspector to every EguiCtx where a
/// InspectorUiRenderTarget also exists
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
#[derive(Clone, Copy, Component, Reflect, Default)]
pub struct InspectorUiRenderTarget;

#[derive(Default)]
pub struct InspectorUiPlugin;

impl Plugin for InspectorUiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, inspector_ui);
	}
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
	let mut query =
		world.query_filtered::<&mut EguiContext, With<InspectorUiRenderTarget>>();

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
