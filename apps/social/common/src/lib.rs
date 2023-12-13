pub mod shared;

use bevy::prelude::*;
use derive_more::{Add, Mul};
use lightyear::inputs::UserInput;
use lightyear::prelude::*;
use lightyear::protocolize;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transports {
	Udp,
}

// Player
#[derive(Bundle)]
pub struct PlayerBundle {
	id: PlayerId,
	position: PlayerPosition,
	color: PlayerColor,
	replicate: Replicate,
	player_avatar_url: PlayerAvatarUrl,
	networked_spatial_audio: NetworkedSpatialAudio,
}

impl PlayerBundle {
	pub fn new(
		id: ClientId,
		position: Vec3,
		color: Color,
		player_avatar_url: PlayerAvatarUrl,
	) -> Self {
		Self {
			id: PlayerId(id),
			position: PlayerPosition(position),
			color: PlayerColor(color),
			replicate: Replicate {
				// prediction_target: NetworkTarget::None,
				prediction_target: NetworkTarget::Only(vec![id]),
				interpolation_target: NetworkTarget::AllExcept(vec![id]),
				..default()
			},
			player_avatar_url,
			networked_spatial_audio: NetworkedSpatialAudio(None),
		}
	}
}

// Components

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub ClientId);

#[derive(
	Component,
	Message,
	Serialize,
	Deserialize,
	Clone,
	Debug,
	PartialEq,
	Deref,
	DerefMut,
	Add,
	Mul,
)]
pub struct PlayerPosition(pub Vec3);

#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerAvatarUrl(pub Option<String>);

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedSpatialAudio(pub Option<Vec<f32>>);

impl Mul<f32> for PlayerAvatarUrl {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		self
	}
}

impl Add for PlayerAvatarUrl {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		self
	}
}

// Example of a component that contains an entity.
// This component, when replicated, needs to have the inner entity mapped from the Server world
// to the client World.
// This can be done by adding a `#[message(custom_map)]` attribute to the component, and then
// deriving the `MapEntities` trait for the component.
#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[message(custom_map)]
pub struct PlayerParent(Entity);

impl MapEntities for PlayerParent {
	fn map_entities(&mut self, entity_map: &EntityMap) {
		self.0.map_entities(entity_map);
	}
}

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
	#[sync(once)]
	PlayerId(PlayerId),
	#[sync(full)]
	PlayerPosition(PlayerPosition),
	#[sync(once)]
	PlayerColor(PlayerColor),
	#[sync(simple)]
	PlayerAvatarUrl(PlayerAvatarUrl),
	#[sync(simple)]
	NetworkedSpatialAudio(NetworkedSpatialAudio),
}

// Channels

#[derive(Channel)]
pub struct Channel1;

#[derive(Channel)]
pub struct AudioChannel;
// Messages

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub String);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MicrophoneAudio(pub Vec<f32>);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerToClientMicrophoneAudio(pub Vec<f32>, pub u64);

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
	Message1(Message1),
	MicrophoneAudio(MicrophoneAudio),
	ServerToClientMicrophoneAudio(ServerToClientMicrophoneAudio),
}

// Inputs

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Direction {
	pub up: bool,
	pub down: bool,
	pub left: bool,
	pub right: bool,
}

impl Direction {
	pub fn is_none(&self) -> bool {
		!self.up && !self.down && !self.left && !self.right
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Inputs {
	Direction(Direction),
	Delete,
	Spawn,
}

impl UserInput for Inputs {}

// Protocol

protocolize! {
	Self = MyProtocol,
	Message = Messages,
	Component = Components,
	Input = Inputs,
}

pub fn protocol() -> MyProtocol {
	let mut protocol = MyProtocol::default();
	protocol.add_channel::<Channel1>(ChannelSettings {
		mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
		direction: ChannelDirection::Bidirectional,
	});
	protocol.add_channel::<AudioChannel>(ChannelSettings {
		mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
		direction: ChannelDirection::Bidirectional,
	});
	protocol
}
