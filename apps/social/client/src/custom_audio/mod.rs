use crate::custom_audio::audio_output::AudioOutputPlugin;
use crate::custom_audio::microphone::MicrophonePlugin;
use bevy::app::PluginGroupBuilder;

pub mod audio_output;
pub mod microphone;
pub mod spatial_audio;

pub struct CustomAudioPlugins;

impl bevy::prelude::PluginGroup for CustomAudioPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(spatial_audio::SpatialAudioPlugin)
			.add(AudioOutputPlugin)
			.add(MicrophonePlugin)
	}
}
