mod player_pose;

pub(crate) use self::player_pose::PlayerPoseInterp;
pub use self::player_pose::{Isometry, PlayerPose};

use bevy::{
	prelude::{App, Color, Component, Entity, Resource},
	reflect::Reflect,
};
use lightyear::prelude::{client::InterpolatedComponent, Message};
use serde::{Deserialize, Serialize};

/// All data in the data model has this entity as its ancestor.
#[derive(Resource, Reflect, Debug)]
pub struct DataModelRoot(pub Entity);

/// Auto-increments for each player that connects.
#[derive(
	Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect,
)]
pub struct PlayerId(u16);

#[derive(
	Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq, Reflect,
)]
pub struct PlayerColor(pub Color);

#[derive(
	Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect,
)]
pub struct PlayerAvatarUrl(pub Option<String>);

#[derive(
	Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect,
)]
pub struct NetworkedSpatialAudio(pub Option<Vec<f32>>);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct MicrophoneAudio(pub Vec<f32>);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct ServerToClientMicrophoneAudio(pub Vec<f32>, pub u64);

pub(crate) fn register_types(app: &mut App) {
	app.register_type::<DataModelRoot>()
		.register_type::<MicrophoneAudio>()
		.register_type::<NetworkedSpatialAudio>()
		.register_type::<PlayerAvatarUrl>()
		.register_type::<PlayerColor>()
		.register_type::<ServerToClientMicrophoneAudio>()
		.register_type::<PlayerId>();
}

// ---- type checks ----

fn _assert_impl_interp() {
	fn check<T: InterpolatedComponent<T>>() {}

	check::<PlayerPose>()
}
