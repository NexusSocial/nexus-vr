use crate::file_sharing::FileParts;
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position};
use bevy::app::App;
use bevy::ecs::system::{EntityCommand, SystemParam};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_matchbox::prelude::{
	MultipleChannels, PeerId, PeerState, WebRtcSocketBuilder,
};
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
	pub peer_id: Option<PeerId>,
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
	Clone, Debug, Serialize, Deserialize, PartialEq, Component,
)]
pub struct RemotePlayer(pub Uuid, pub PeerId);

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
		app.add_event::<SpawnPlayer>();
		app.add_event::<PlayerConnected>();
		app.add_event::<PlayerDisconnected>();
		app.add_event::<UpdatePhysicsPosition>();

		app.add_systems(
			FixedUpdate,
			update_peers.run_if(resource_exists::<MatchboxSocket<MultipleChannels>>),
		);
		app.add_systems(
			Update,
			(handle_messages_reliable, handle_messages_unreliable),
		);
	}
}

fn handle_messages_reliable(
	mut connection: Connection,
	mut spawn_players: EventWriter<SpawnPlayer>,
	mut file_parts: ResMut<FileParts>,
) {
	for (message, peer_id) in connection.recv() {
		let message: ReliableMessage = message;
		match message {
			ReliableMessage::SpawnPlayer(mut spawn_player) => {
				spawn_player.peer_id.replace(peer_id);
				spawn_players.send(spawn_player);
			}
			ReliableMessage::FilePart(file_part) => {
				file_parts.0.push(file_part);
			}
		}
	}
}

fn handle_messages_unreliable(
	mut connection: Connection,
	mut update_physics_positions: EventWriter<UpdatePhysicsPosition>,
) {
	for (message, _peer_id) in connection.recv() {
		let message: UnreliableMessage = message;
		match message {
			UnreliableMessage::UpdatePhysicsPosition(update_physics_position) => {
				update_physics_positions.send(update_physics_position);
			}
			UnreliableMessage::PlayerPositionUpdate(_) => todo!(),
		}
	}
}

#[derive(Event)]
pub struct PlayerDisconnected(pub PeerId);

#[derive(Event)]
pub struct PlayerConnected(pub PeerId);

fn update_peers(
	mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
	mut player_connected: EventWriter<PlayerConnected>,
	mut player_disconnected: EventWriter<PlayerDisconnected>,
) {
	for (peer, state) in socket.update_peers() {
		match state {
			PeerState::Connected => {
				player_connected.send(PlayerConnected(peer));
			}
			PeerState::Disconnected => {
				player_disconnected.send(PlayerDisconnected(peer));
			}
		}
	}
}

#[derive(SystemParam)]
pub struct Connection<'w> {
	socket: ResMut<'w, MatchboxSocket<MultipleChannels>>,
}

pub trait ConnectionTrait<MessageType> {
	fn peers(&self) -> Vec<PeerId>;
	fn update_peers(&mut self) -> Vec<(PeerId, PeerState)>;
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

	fn recv(&mut self) -> Vec<(MessageType, PeerId)>;
}

impl ConnectionTrait<UnreliableMessage> for Connection<'_> {
	fn peers(&self) -> Vec<PeerId> {
		self.socket.connected_peers().collect::<Vec<_>>()
	}

	fn update_peers(&mut self) -> Vec<(PeerId, PeerState)> {
		self.socket.update_peers()
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

	fn recv(&mut self) -> Vec<(UnreliableMessage, PeerId)> {
		self.socket
			.channel_mut(1)
			.receive()
			.into_iter()
			.map(|(peer_id, packet)| (bincode::deserialize(&packet).unwrap(), peer_id))
			.collect()
	}
}

impl ConnectionTrait<ReliableMessage> for Connection<'_> {
	fn peers(&self) -> Vec<PeerId> {
		self.socket.connected_peers().collect::<Vec<_>>()
	}

	fn update_peers(&mut self) -> Vec<(PeerId, PeerState)> {
		self.socket.update_peers()
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

	fn recv(&mut self) -> Vec<(ReliableMessage, PeerId)> {
		self.socket
			.channel_mut(0)
			.receive()
			.into_iter()
			.map(|(peer_id, packet)| (bincode::deserialize(&packet).unwrap(), peer_id))
			.collect()
	}
}
