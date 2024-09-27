use avian3d::prelude::{AngularVelocity, LinearVelocity, Position};
use bevy::app::App;
use bevy::ecs::system::{EntityCommand, SystemParam};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_matchbox::prelude::{MultipleChannels, PeerId, WebRtcSocketBuilder};
use bevy_matchbox::MatchboxSocket;
use futures_channel::mpsc::SendError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReliableMessage {
	SpawnPlayer(SpawnPlayer),
	FilePart(crate::file_sharing::FilePart),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnreliableMessage {
	UpdatePhysicsPosition(UpdatePhysicsPosition),
	PlayerPositionUpdate(PlayerPositionUpdate),
}

#[derive(Clone, Debug, Serialize, Deserialize, Event)]
pub struct SpawnPlayer {
	pub uuid: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, Event)]
pub struct UpdatePhysicsPosition {
	pub authority: u64,
	pub uuid: Uuid,
	pub position: Position,
	pub rotation: avian3d::position::Rotation,
	pub linear_velocity: LinearVelocity,
	pub angular_velocity: AngularVelocity,
}

#[derive(Clone, Debug, Serialize, Deserialize, Event)]
pub struct PlayerPositionUpdate {
	uuid: Uuid,
	head: Transform,
	left_upper_arm: Transform,
	right_upper_arm: Transform,
	left_lower_arm: Transform,
	right_lower_arm: Transform,
	left_hand: Transform,
	right_hand: Transform,
}

#[derive(
	Reflect, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, PartialEq, Component,
)]
pub struct LocalPlayer(pub Uuid);

#[derive(
	Reflect, Clone, Debug, Serialize, Deserialize, Deref, DerefMut, PartialEq, Component,
)]
pub struct RemotePlayer(pub Uuid);

pub trait ConnectToRoom {
	fn connect_to_room(&mut self, room: impl AsRef<str>);
}

impl ConnectToRoom for Commands<'_, '_> {
	fn connect_to_room(&mut self, room: impl AsRef<str>) {
		let matchbox_socket = MatchboxSocket::from(
			WebRtcSocketBuilder::new(format!(
				"wss://mb.v-sekai.cloud/{}",
				room.as_ref()
			))
			.add_reliable_channel()
			.add_unreliable_channel(),
		);
		self.insert_resource(matchbox_socket);
	}
}

pub struct NetworkingPlugin;
impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			FixedUpdate,
			update_peers.run_if(resource_exists::<MatchboxSocket<MultipleChannels>>),
		);
	}
}

fn update_peers(mut socket: ResMut<MatchboxSocket<MultipleChannels>>) {
	socket.update_peers();
}

#[derive(SystemParam)]
pub struct Connection<'w> {
	socket: ResMut<'w, MatchboxSocket<MultipleChannels>>,
}

pub trait ConnectionTrait<MessageType> {
	fn peers(&self) -> Vec<PeerId>;
	fn update_peers(&mut self);
	fn send(
		&mut self,
		peer: bevy_matchbox::prelude::PeerId,
		message: &MessageType,
	) -> Result<(), futures_channel::mpsc::SendError>;
	fn send_all(
		&mut self,
		message: &MessageType,
	) -> Result<(), Vec<(PeerId, futures_channel::mpsc::SendError)>> {
		// here so we don't allocate on the heap unless there's an error
		let mut possible_vec = None;

		for peer in self.peers() {
			if let Err(err) = self.send(peer, message) {
				if possible_vec.is_none() {
					possible_vec.replace(vec![]);
				}
				possible_vec.as_mut().unwrap().push((peer, err));
			}
		}

		match possible_vec {
			None => Ok(()),
			Some(vec) => Err(vec),
		}
	}
}

impl ConnectionTrait<UnreliableMessage> for Connection<'_> {
	fn peers(&self) -> Vec<PeerId> {
		self.socket.connected_peers().collect::<Vec<_>>()
	}

	fn update_peers(&mut self) {
		self.socket.update_peers();
	}

	fn send(
		&mut self,
		peer: PeerId,
		message: &UnreliableMessage,
	) -> Result<(), SendError> {
		let encoded = bincode::serialize(message).unwrap();
		self.socket
			.channel_mut(1)
			.try_send(Box::from(encoded), peer)
	}
}

impl ConnectionTrait<ReliableMessage> for Connection<'_> {
	fn peers(&self) -> Vec<PeerId> {
		self.socket.connected_peers().collect::<Vec<_>>()
	}

	fn update_peers(&mut self) {
		self.socket.update_peers();
	}

	fn send(
		&mut self,
		peer: PeerId,
		message: &ReliableMessage,
	) -> Result<(), SendError> {
		let encoded = bincode::serialize(message).unwrap();
		self.socket
			.channel_mut(0)
			.try_send(Box::from(encoded), peer)
	}
}
