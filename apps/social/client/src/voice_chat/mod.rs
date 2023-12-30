use crate::custom_audio::microphone::MicrophoneAudio;
use crate::custom_audio::spatial_audio::SpatialAudioSink;
use bevy::app::{App, PreUpdate};
use bevy::prelude::{
	EventReader, EventWriter, Local, NonSendMut, Query, ResMut, Resource, Update, With,
};
use opus::{Application, Channels, Decoder, Encoder};
use social_networking::client::{ClientToServerVoiceMsg, ServerToClientVoiceMsg};
use social_networking::data_model::ClientIdComponent;

pub struct VoiceChatPlugin;

impl bevy::prelude::Plugin for VoiceChatPlugin {
	fn build(&self, app: &mut App) {
		app.insert_non_send_resource(MicrophoneEncoder(
			Encoder::new(48_000, Channels::Mono, Application::Voip)
				.expect("unable to create microphone audio compressing encoder"),
		));
		app.insert_non_send_resource(MicrophoneDecoder(
			Decoder::new(48_000, Channels::Mono)
				.expect("unable to create microphone audio compressing decoder"),
		));
		app.add_systems(Update, send_voice_msg);
		app.add_systems(Update, rec_voice_msg);
		app.add_systems(Update, bad_jitter_buffer);
	}
}

#[derive(Resource)]
pub struct MicrophoneEncoder(pub Encoder);
#[derive(Resource)]
pub struct MicrophoneDecoder(pub Decoder);
unsafe impl Sync for MicrophoneEncoder {}
unsafe impl Sync for MicrophoneDecoder {}

fn send_voice_msg(
	mut encoder: NonSendMut<MicrophoneEncoder>,
	mut microphone: ResMut<MicrophoneAudio>,
	mut event_writer: EventWriter<ClientToServerVoiceMsg>,
	mut local_size: Local<Vec<f32>>,
) {
	while let Ok(mut audio) = microphone.0.lock().unwrap().try_recv() {
		local_size.append(&mut audio);
	}
	if local_size.len() < 2880 {
		return;
	}
	while local_size.len() > 2880 {
		event_writer.send(ClientToServerVoiceMsg(
			encoder
				.0
				.encode_vec_float(local_size.drain(0..2880).as_ref(), 2880)
				.expect("couldnt' encode audio"),
		))
	}
}

fn rec_voice_msg(
	mut event_reader: EventReader<ServerToClientVoiceMsg>,
	mut microphone_decoder: NonSendMut<MicrophoneDecoder>,
	mut players: Query<(&ClientIdComponent, &mut SpatialAudioSink)>,
) {
	for event in event_reader.read() {
		let id2 = event.0;
		let audio = &event.1;
		for (id, audio_sink) in players.iter_mut() {
			if id2 != id.0 {
				continue;
			}
			let mut output = [0.0; 2880];
			microphone_decoder
				.0
				.decode_float(audio, &mut output, false)
				.expect("unable to decode audio");
			audio_sink
				.sink
				.append(rodio::buffer::SamplesBuffer::new(1, 48_000, output));
			audio_sink.sink.play();
			audio_sink.sink.set_volume(1.0);
		}
	}
}

fn bad_jitter_buffer(
	mut spatial_audio_sinks: Query<&mut SpatialAudioSink, With<ClientIdComponent>>,
) {
	for spatial_audio_sink in spatial_audio_sinks.iter_mut() {
		if spatial_audio_sink.sink.len() <= 1 {
			spatial_audio_sink.sink.pause();
			continue;
		} else {
			spatial_audio_sink.sink.play();
		}
		if spatial_audio_sink.sink.len() <= 3 {
			spatial_audio_sink.sink.set_speed(0.95);
		} else if spatial_audio_sink.sink.len() >= 10 {
			spatial_audio_sink
				.sink
				.set_speed(spatial_audio_sink.sink.len() as f32 / 10.0);
		} else {
			spatial_audio_sink.sink.set_speed(1.0);
		}
	}
}
