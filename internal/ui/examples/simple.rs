use bevy::prelude::*;
use bevy_egui::{EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use nx_ui::{UiViewPlugin, UiViewT, UiViewTitle};

#[derive(Component, Debug)]
#[require(UiViewTitle::new("My UI"))]
struct MyUi;

impl UiViewT for MyUi {
	fn render_ui(&mut self, ui: &mut egui::Ui) {
		ui.label("my ui here");
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(EguiPlugin::default())
		.add_plugins(WorldInspectorPlugin::default())
		.add_plugins(UiViewPlugin::<MyUi>::default())
		.add_systems(Startup, setup)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn((MyUi, PrimaryEguiContext));
}
