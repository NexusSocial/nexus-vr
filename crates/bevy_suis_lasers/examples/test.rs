use bevy::prelude::*;
use bevy_mod_openxr::add_xr_plugins;
use bevy_suis::{
	debug::SuisDebugGizmosPlugin, window_pointers::SuisWindowPointerPlugin,
	xr_controllers::SuisXrControllerPlugin, CaptureContext, Field, InputHandler,
	SuisCorePlugin,
};
use bevy_suis_lasers::{
	draw_lasers::LaserPlugin, laser_input_methods::LaserInputMethodPlugin,
};
fn main() -> AppExit {
	App::new()
		.add_plugins(add_xr_plugins(DefaultPlugins))
		.add_plugins((
			SuisCorePlugin,
			SuisXrControllerPlugin,
			SuisWindowPointerPlugin,
			SuisDebugGizmosPlugin,
		))
		.add_plugins((LaserPlugin, LaserInputMethodPlugin))
		.add_systems(Startup, setup)
		.run()
}

fn setup(mut cmds: Commands) {
	cmds.spawn(SpatialBundle::from_transform(Transform::from_xyz(
		0.0, 1.0, -2.0,
	)))
	.insert(Field::Cuboid(Cuboid::from_size(Vec3::splat(0.25))))
	.insert(InputHandler::new(|_: In<CaptureContext>| true));
}
