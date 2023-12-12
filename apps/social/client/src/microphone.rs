use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::buffer::SamplesBuffer;
use rodio::queue::SourcesQueueOutput;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct MicrophonePlugin;

impl Plugin for MicrophonePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, on_startup_add_microphone);
		app.add_systems(Update, play_microphone);
	}
}

#[derive(Component)]
struct MicrophonePlaying;

#[derive(Component)]
struct SinkWrapper(pub Sink);
fn play_microphone(
	mut microphone_output: ResMut<MicrophoneOutput>,
	mut commands: Commands,
	mut query: Query<&mut SinkWrapper, With<MicrophonePlaying>>,
) {
	if query.is_empty() {
		let stream_handle1 = Arc::new(Mutex::new(None));
		let sh = stream_handle1.clone();
		thread::spawn(move || {
			let (_stream, stream_handle) = OutputStream::try_default().unwrap();
			stream_handle1.lock().unwrap().replace(stream_handle);
			loop {
				thread::sleep(Duration::from_secs(100));
			}
		});
		thread::sleep(Duration::from_secs(1));
		let s = sh.lock().unwrap();
		let stream_handle = s.as_ref().unwrap();
		let sink = Sink::try_new(&stream_handle).unwrap();
		commands.spawn((SinkWrapper(sink), MicrophonePlaying));
		return;
	}
	let q = query.get_single_mut().unwrap();
	let mut i = 0;
	for audio in microphone_output.0.lock().unwrap().iter() {
		q.0.append(SamplesBuffer::new(1, 44100, audio));
		q.0.set_volume(1.0);

		q.0.set_speed(1.0);

		q.0.play();
	}
}

#[derive(Resource)]
pub struct MicrophoneOutput(pub Mutex<Receiver<Vec<f32>>>);

fn on_startup_add_microphone(mut commands: Commands) {
	let (tx, rx) = channel();
	commands.insert_resource(MicrophoneOutput(Mutex::new(rx)));
	let channels = 1;
	std::thread::spawn(move || {
		let device = cpal::default_host().default_input_device().unwrap();
		let mut configs = device.supported_input_configs().unwrap();
		let config = configs
			.find(|c| {
				c.sample_format() == cpal::SampleFormat::F32 && c.channels() == channels
				// && c.min_sample_rate().0 < mic_config.sample_rate
				// && c.max_sample_rate().0 > mic_config.sample_rate
			})
			.unwrap()
			.with_sample_rate(cpal::SampleRate(44_100));
		let err_fn =
			|err| error!("an error occurred on the output audio stream: {}", err);
		let stream = device
			.build_input_stream(
				&config.into(),
				move |d: &[f32], _| {
					//println!("{:?}", d);
					for slice in d.chunks(channels as _) {
						tx.send((slice).to_vec()).unwrap();
					}
				},
				err_fn,
				None,
			)
			.unwrap();
		stream.play().unwrap();
		loop {
			std::thread::sleep(Duration::from_secs(100));
		}
	});
}
