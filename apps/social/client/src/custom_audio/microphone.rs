use bevy::app::App;
use bevy::log::{debug, error};
#[allow(deprecated)]
use bevy::prelude::{warn, Commands, Resource, Startup};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;

pub struct MicrophonePlugin;

impl bevy::prelude::Plugin for MicrophonePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, create_microphone);
	}
}

#[derive(Resource)]
pub struct MicrophoneAudio(pub Mutex<Receiver<Vec<f32>>>);

#[derive(Debug)]
pub struct MicrophoneConfig {
	channels: u16,
	sample_rate: u32,
}

impl Default for MicrophoneConfig {
	fn default() -> Self {
		Self {
			channels: 1,
			sample_rate: 48_000,
		}
	}
}

pub fn create_microphone(mut commands: Commands) {
	#[allow(unused_mut)]
	let mut microphone_config = MicrophoneConfig::default();

	// we wanna share the output from our thread loop thing in here continuously with the rest of bevy.
	let (tx, rx) = channel();
	commands.insert_resource(MicrophoneAudio(Mutex::new(rx)));

	std::thread::spawn(move || {
		// Setup microphone device
		let device = match cpal::default_host().default_input_device() {
			None => return warn!("no audio input device found, microphone functionality will be disabled"),
			Some(device) => device,
		};
		let configs = match device.supported_input_configs() {
			Ok(configs) => configs,
			Err(err) => return warn!("supported stream config error, microphone functionality will be disabled, error: {}", err),
		};
		for config in configs {
			debug!("supported microphone config: {:#?}", config);
		}
		let mut configs = match device.supported_input_configs() {
			Ok(configs) => configs,
			Err(err) => return warn!("supported stream config error, microphone functionality will be disabled, error: {}", err),
		};

		#[cfg(target_os = "android")]
		{
			microphone_config.channels = 2;
		}

		let config = match configs
            .find(|c| {
                c.sample_format() == cpal::SampleFormat::F32 && c.channels() == microphone_config.channels
                 && c.min_sample_rate().0 <= microphone_config.sample_rate
                 && c.max_sample_rate().0 >= microphone_config.sample_rate
            })
        {
            None => return warn!("microphone config of {:?} not supported, microphone functionality will be disabled", microphone_config),
            Some(config) => config,
        }
            .with_sample_rate(cpal::SampleRate(microphone_config.sample_rate));

		// Run microphone audio through our channel
		let err_fn =
			|err| error!("an error occurred on the output audio stream: {}", err);
		let stream = device
			.build_input_stream(
				&config.into(),
				move |d: &[f32], _| {
					tx.send(d.to_vec()).unwrap();
				},
				err_fn,
				None,
			)
			.unwrap();

		// we play the stream, and then in order to not drop the stuff we have here, to continue to play it continously
		// we have to loop, we sleep for 100 seconds so this thread rarely ever does anything.
		stream.play().unwrap();
		loop {
			std::thread::sleep(std::time::Duration::from_secs(100));
		}
	});
}
