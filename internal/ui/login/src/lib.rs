use bevy::ecs::component::Component;
use nx_ui::{UiViewT, UiViewTitle};

// Should it be a resource?
#[derive(Debug, Default, Component)]
#[require(UiViewTitle::new("Login UI"))]
pub struct LoginUi;

impl UiViewT for LoginUi {
	fn render_ui(&mut self, ui: &mut egui::Ui) {
		ui.label("foobar");
	}
}
