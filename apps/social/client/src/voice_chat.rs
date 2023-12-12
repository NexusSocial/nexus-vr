use crate::microphone::MicrophoneOutput;
use crate::networking::PlayerClientId;
use bevy::prelude::*;
use lightyear::prelude::client::{Client, MessageEvent};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStreamHandle, SpatialSink};
use social_common::{
	AudioChannel, Channel1, MicrophoneAudio, MyProtocol, NetworkedSpatialAudio,
	PlayerId, ServerToClientMicrophoneAudio,
};

pub struct VoiceChatPlugin;

impl Plugin for VoiceChatPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, set_spatial_audio_pos);
		//app.add_systems(Startup, spawn_boi);
		app.insert_resource(AudioOutput::default());
		app.add_systems(Update, play_microphone);
		app.add_systems(Update, on_player_add);
		app.add_systems(Update, sync_spatial_audio);
	}
}

#[derive(Resource)]
pub struct AudioOutput {
	stream_handle: Option<OutputStreamHandle>,
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
			warn!("No audio device found.");
			Self {
				stream_handle: None,
			}
		}
	}
}

#[derive(Component)]
pub struct SpatialAudioSink {
	pub sink: rodio::SpatialSink,
}

fn on_player_add(
	mut commands: Commands,
	mut query: Query<(Entity, &PlayerId), Added<PlayerId>>,
	player_client_id: Res<PlayerClientId>,
	audio_output: Res<AudioOutput>,
) {
	let Some(stream_handle) = audio_output.stream_handle.as_ref() else {
		return;
	};
	for (entity, player_id) in query.iter() {
		// we don't wanna do audio for our own player
		if player_id.0 == player_client_id.0 {
			//continue;
		}
		commands.entity(entity).insert(SpatialAudioSink {
			sink: SpatialSink::try_new(
				stream_handle,
				Vec3::new(0.0, 0.0, 0.0).to_array(),
				(Vec3::X * 4.0 / -2.0).to_array(),
				(Vec3::X * 4.0 / 2.0).to_array(),
			)
			.unwrap(),
		});
	}
}

fn set_spatial_audio_pos(
	mut query: Query<(&mut SpatialAudioSink, &Transform), Changed<Transform>>,
) {
	for (mut spatial_audio_sink, transform) in query.iter_mut() {
		spatial_audio_sink
			.sink
			.set_emitter_position(transform.translation.to_array());
	}
}

fn play_microphone(
	mut microphone_output: ResMut<MicrophoneOutput>,
	//mut query: Query<(&mut SpatialAudioSink)>,
	mut client: ResMut<Client<MyProtocol>>,
	mut local: Local<Vec<f32>>,
) {
	for mut audio in microphone_output.0.lock().unwrap().try_iter() {
		/*client
		.buffer_send::<AudioChannel, _>(MicrophoneAudio(audio))
		.unwrap();*/
		local.append(&mut audio);
	}
	if local.is_empty() {
		return;
	}
	if local.len() < 20_000 {
		return;
	}
	client
		.buffer_send::<AudioChannel, _>(MicrophoneAudio(local.to_vec()))
		.unwrap();
	local.clear();
}

fn sync_spatial_audio(
	mut audio_msg: EventReader<MessageEvent<ServerToClientMicrophoneAudio>>,
	mut query: Query<(&PlayerId, &mut SpatialAudioSink)>,
) {
	for audio in audio_msg.read() {
		let audio = audio.message();
		let id = audio.1.clone();
		for (player_id, mut spatial_audio_sink) in query.iter_mut() {
			if player_id.0 != id {
				continue;
			}
			spatial_audio_sink.sink.append(SamplesBuffer::new(
				1,
				44100,
				audio.0.clone(),
			));
			spatial_audio_sink.sink.set_volume(1.0);
			//spatial_audio_sink.sink.play();
		}
	}
	for (_, mut spatial_audio_sink) in query.iter_mut() {
		println!("{}", spatial_audio_sink.sink.len());
		if spatial_audio_sink.sink.len() < 2 {
			spatial_audio_sink.sink.pause();
		} else if spatial_audio_sink.sink.len() > 6 {
			spatial_audio_sink.sink.play();
		}
		// if spatial_audio_sink.sink.len() < 6 {
		// 	spatial_audio_sink
		// 		.sink
		// 		.set_speed((spatial_audio_sink.sink.len() as f32) / 6.0);
		// } else {
		// 	spatial_audio_sink.sink.set_speed(1.0);
		// }
	}
}

fn spawn_boi(mut commands: Commands, audio_output: Res<AudioOutput>) {
	let output = match audio_output.stream_handle.as_ref() {
		None => return,
		Some(output) => output,
	};
	commands.spawn((
		TransformBundle::from_transform(Transform::from_xyz(1.0, 0.0, 0.0)),
		SpatialAudioSink {
			sink: SpatialSink::try_new(
				output,
				Vec3::new(1.0, 0.0, 0.0).to_array(),
				(Vec3::X * 4.0 / -2.0).to_array(),
				(Vec3::X * 4.0 / 2.0).to_array(),
			)
			.unwrap(),
		},
	));
}
