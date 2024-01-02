use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

use bevy::app::AppExit;
use bevy::prelude::App;
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use color_eyre::eyre::Context;
use color_eyre::Result;
use social_client::MainPlugin;

fn main() -> Result<()> {
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

fn kill_after_frames(
	mut n_frame: Local<u64>,
	mut scshot: ResMut<ScreenshotManager>,
	main_window: Query<Entity, With<PrimaryWindow>>,
	mut exit_evts: EventWriter<AppExit>,
	mut recv_chan: Local<Option<mpsc::Receiver<Result<()>>>>,
) {
	if let Some(recv_chan) = recv_chan.as_mut() {
		match recv_chan.try_recv() {
			Ok(Ok(())) => exit_evts.send_default(),
			Ok(Err(err)) => panic!("{err:?}"),
			Err(TryRecvError::Disconnected) => panic!("disconnected"),
			Err(TryRecvError::Empty) => {}
		}
		return;
	}
	if *n_frame >= 30 {
		let (send, recv) = mpsc::channel();
		*recv_chan = Some(recv);
		let cb = move |img: Image| {
			let filename = "render_test-screenshot.png";
			let screenshots_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests")
				.join("screenshots");
			let file_path = screenshots_dir.join(filename);
			let result = img
				.try_into_dynamic()
				.wrap_err("failed to convert to DynamicImage")
				.and_then(|img| {
					img.save(&file_path).wrap_err(format!(
						"failed to save screenshot to disk with path {}",
						file_path.display()
					))
				});
			info!("saved image to {}", file_path.display());
			send.send(result).expect("could not send?");
		};
		scshot
			.take_screenshot(main_window.single(), cb)
			.expect("err while saving screenshot");
	}
	*n_frame += 1;
}
