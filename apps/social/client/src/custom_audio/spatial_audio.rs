use bevy::app::App;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::{
	warn, Bundle, Changed, Component, GlobalTransform, Query, SpatialBundle, Update,
	With,
};

pub struct SpatialAudioPlugin;

impl bevy::prelude::Plugin for SpatialAudioPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, set_spatial_audio_sink_pos);
	}
}

#[derive(Component)]
pub struct SpatialAudioSink {
	pub sink: rodio::SpatialSink,
}

/// There should only ever be one spatial audio listener
#[derive(Component, Debug)]
pub struct SpatialAudioListener;

#[derive(Bundle)]
pub struct SpatialAudioSinkBundle {
	pub(crate) spatial_audio_sink: SpatialAudioSink,
	pub(crate) spatial_bundle: SpatialBundle,
}

/// There should only ever be one spatial audio listener
#[derive(Bundle)]
pub struct SpatialAudioListenerBundle {
	pub(crate) spatial_audio_listener: SpatialAudioListener,
	pub(crate) spatial_bundle: SpatialBundle,
}

fn set_spatial_audio_sink_pos(
	mut spatial_audio_sinks: Query<
		(&mut SpatialAudioSink, &GlobalTransform),
		Changed<GlobalTransform>,
	>,
	spatial_audio_listener: Query<&GlobalTransform, With<SpatialAudioListener>>,
) {
	let spatial_audio_listener = match spatial_audio_listener.get_single() {
		Ok(spatial_audio_listener) => spatial_audio_listener,
		Err(err) => match err {
			QuerySingleError::NoEntities(_) => {
				warn!("no spatial audio listener, spatial audio sinks are placed at 0,0,0 by default");
				return;
			}
			QuerySingleError::MultipleEntities(_) => {
				panic!("there should only ever be one spatial audio listener at a time")
			}
		},
	};
	let spatial_audio_listener_translation = spatial_audio_listener.translation();
	for (mut spatial_audio_sink, global_transform) in spatial_audio_sinks.iter_mut() {
		let spatial_audio_sink_translation = global_transform.translation();
		let emitter_position =
			spatial_audio_listener_translation - spatial_audio_sink_translation;
		spatial_audio_sink
			.sink
			.set_emitter_position(emitter_position.to_array());
	}
}
