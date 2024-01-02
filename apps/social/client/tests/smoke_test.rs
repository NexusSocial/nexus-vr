use bevy::app::AppExit;
use bevy::prelude::App;
use bevy::prelude::*;
use social_client::MainPlugin;

fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;

	let mut app = App::new();
	app.add_plugins(MainPlugin {
		exec_mode: social_client::ExecutionMode::Normal,
		server_addr: None,
	})
	.add_systems(Update, kill_after_frames);
	app.run();

	Ok(())
}

fn kill_after_frames(mut n_frame: Local<u8>, mut exit_evts: EventWriter<AppExit>) {
	if *n_frame >= 60 {
		exit_evts.send_default();
	}
	*n_frame += 1;
}
