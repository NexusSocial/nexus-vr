use bevy::{
	app::PluginGroupBuilder,
	prelude::{default, App, DefaultPlugins, PluginGroup},
	render::{settings::Backends, RenderPlugin},
	winit::WinitPlugin,
};

/// Makes an bevy app for testing.
/// `interactive_setup` will be called if the app is running on the main thread
/// (i.e. not in a regular test runner).
///
/// See also, [`HeadlessRenderPlugins`] for an easy way to set bevy up headlessly.
pub fn make_test_app(interactive_setup: impl FnOnce(&mut App)) -> App {
	let mut app = App::new();
	if on_main_thread() {
		interactive_setup(&mut app);
	}
	app
}

pub fn on_main_thread() -> bool {
	matches!(std::thread::current().name(), Some("main"))
}

pub struct HeadlessRenderPlugins;

impl PluginGroup for HeadlessRenderPlugins {
	fn build(self) -> PluginGroupBuilder {
		DefaultPlugins
			.set(RenderPlugin {
				render_creation: bevy::render::settings::RenderCreation::Automatic(
					bevy::render::settings::WgpuSettings {
						backends: Some(Backends::empty()),
						..default()
					},
				),
			})
			.disable::<WinitPlugin>()
	}
}
