use bevy::app::App;
use bevy::prelude::Resource;

pub struct AudioOutputPlugin;

impl bevy::prelude::Plugin for AudioOutputPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(AudioOutput::default());
	}
}

#[derive(Resource)]
pub struct AudioOutput {
	pub stream_handle: Option<rodio::OutputStreamHandle>,
}

impl Default for AudioOutput {
	fn default() -> Self {
		if let Ok((stream, stream_handle)) = rodio::OutputStream::try_default() {
			// We leak `OutputStream` to prevent the audio from stopping.
			std::mem::forget(stream);
			Self {
				stream_handle: Some(stream_handle),
			}
		} else {
			bevy::log::warn!("No audio output device found.");
			Self {
				stream_handle: None,
			}
		}
	}
}
