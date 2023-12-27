use crate::microphone::MicrophoneOutput;
use crate::networking::PlayerClientId;
use bevy::prelude::*;
use lightyear::prelude::client::{Client, MessageEvent, Predicted};
use rodio::buffer::SamplesBuffer;

use rodio::{OutputStreamHandle, SpatialSink};
use social_common::{
	AudioChannel, MicrophoneAudio, MyProtocol, PlayerId, ServerToClientMicrophoneAudio,
};

pub struct VoiceChatPlugin;

impl Plugin for VoiceChatPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, set_spatial_audio_pos);
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
	query: Query<(Entity, &PlayerId), (Added<PlayerId>, With<Predicted>)>,
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
	player_client_id: Res<PlayerClientId>,
	mut query: Query<
		(&PlayerId, &mut SpatialAudioSink, &GlobalTransform),
		Changed<GlobalTransform>,
	>,
) {
	let mut player_transform = Vec3::default();
	for (player_id, _, transform) in query.iter() {
		if player_id.0 == player_client_id.0 {
			player_transform = transform.translation();
			break;
		}
	}

	for (_, spatial_audio_sink, transform) in query.iter_mut() {
		let t = player_transform - transform.translation();
		println!(
			"player transform: {}, relative_transform: {}",
			player_transform, t
		);
		spatial_audio_sink.sink.set_emitter_position(t.to_array());
	}
}

fn play_microphone(
	microphone_output: ResMut<MicrophoneOutput>,
	//mut query: Query<(&mut SpatialAudioSink)>,
	mut client: ResMut<Client<MyProtocol>>,
	_local: Local<Vec<f32>>,
) {
	for audio in microphone_output.0.lock().unwrap().try_iter() {
		client
			.send_message::<AudioChannel, _>(MicrophoneAudio(audio))
			.unwrap();
		//local.append(&mut audio);
	}
	// if local.is_empty() {
	// 	return;
	// }
	// if local.len() < 1_000 {
	// 	return;
	// }
	// client
	// 	.buffer_send::<AudioChannel, _>(MicrophoneAudio(local.to_vec()))
	// 	.unwrap();
	// local.clear();
}

fn sync_spatial_audio(
	mut audio_msg: EventReader<MessageEvent<ServerToClientMicrophoneAudio>>,
	mut query: Query<(&PlayerId, &mut SpatialAudioSink)>,
	mut local: Local<Vec<f32>>,
) {
	for audio in audio_msg.read() {
		let audio = audio.message();
		let id = audio.1;
		let mut clear_local = false;
		local.append(&mut audio.0.clone());
		for (player_id, spatial_audio_sink) in query.iter_mut() {
			if player_id.0 != id {
				continue;
			}
			if local.len() > 10_000 {
				spatial_audio_sink.sink.append(SamplesBuffer::new(
					1,
					44100,
					local.clone(),
				));
				clear_local = true;
			}
			/*spatial_audio_sink.sink.set_volume(1.0);*/
		}
		if clear_local {
			local.clear();
			println!("adding");
		}
	}
	for (_, spatial_audio_sink) in query.iter_mut() {
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
