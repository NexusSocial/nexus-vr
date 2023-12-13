use bevy::prelude::*;
use lightyear::prelude::server::*;
use social_common::{
	AudioChannel, MicrophoneAudio, MyProtocol, NetworkedSpatialAudio, PlayerId,
	ServerToClientMicrophoneAudio,
};

pub struct VoiceChatPlugin;

impl Plugin for VoiceChatPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, receive_voice_chat_audio);
	}
}

fn receive_voice_chat_audio(
	mut messages: EventReader<MessageEvent<MicrophoneAudio>>,
	/*mut query: Query<(&PlayerId, &mut NetworkedSpatialAudio)>,*/
	mut server: ResMut<Server<MyProtocol>>,
) {
	for message in messages.read() {
		let id2 = *message.context();
		let audio = message.message();
		for id in server.client_ids().collect::<Vec<_>>() {
			server
				.buffer_send::<AudioChannel, _>(
					id,
					ServerToClientMicrophoneAudio(audio.0.clone(), id2),
				)
				.unwrap();
		}
		/*for (player_id, mut networked_spatial_audio) in query.iter_mut() {
			// we don't wanna send audio back to ourselves.
			if player_id.0 == id {
				//continue;
			}

			networked_spatial_audio.0.replace(audio.0.clone());
		}*/
	}
}
