mod entity_mapper;
mod player_pose;

pub(crate) use self::player_pose::PlayerPoseInterp;
pub use self::player_pose::{Isometry, PlayerPose};

use bevy::{
	prelude::{App, Bundle, Component, Entity, Resource},
	reflect::Reflect,
};
use lightyear::prelude::{client::InterpolatedComponent, Message};
use serde::{Deserialize, Serialize};

/// Data model bundle for avatars.
#[derive(Bundle, Debug, Default)]
pub struct AvatarBundle {
	//pub avi_url: PlayerAvatarUrl,
	pub pose: PlayerPose,
}

/// Entities with this property are not written from the server. Instead, other plugins
/// should write into the data model themselves, and this plugin will sync that info
/// back to the server.
// TODO: How should we handle conflict between server and local, if its server auth?
#[derive(Component, Debug, Reflect, Default)]
pub struct Local;

/// All data in the data model has this entity as its ancestor.
#[derive(Resource, Reflect, Debug)]
pub struct DataModelRoot(pub Entity);

#[derive(
	Component,
	Message,
	Serialize,
	Deserialize,
	Clone,
	Debug,
	PartialEq,
	Reflect,
	Default,
)]
pub struct PlayerAvatarUrl(pub String);

#[derive(
	Component,
	Message,
	Serialize,
	Deserialize,
	Clone,
	Debug,
	PartialEq,
	Reflect,
	Default,
)]
pub struct Avatar;

#[derive(
	Component,
	Message,
	Serialize,
	Deserialize,
	Clone,
	Debug,
	PartialEq,
	Reflect,
	Default,
)]
pub struct ClientIdComponent(pub lightyear::netcode::ClientId);

pub(crate) fn register_types(app: &mut App) {
	app.register_type::<Local>()
		.register_type::<PlayerAvatarUrl>()
		.register_type::<DataModelRoot>()
		.register_type::<PlayerPose>();
}

// ---- type checks ----

fn _assert_impl_interp() {
	fn check<T: InterpolatedComponent<T>>() {}

	check::<PlayerPose>()
}
