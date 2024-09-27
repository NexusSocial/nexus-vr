use crate::networking::{Connection, ConnectionTrait, LocalPlayer, PlayerConnected, PlayerDisconnected, ReliableMessage, RemotePlayer, SpawnPlayer, UnreliableMessage, UpdatePhysicsPosition};
use crate::spawn_avatar;
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use bevy_matchbox::prelude::MultipleChannels;
use bevy_matchbox::MatchboxSocket;
use bevy_vr_controller::animation::defaults::default_character_animations;
use bevy_vr_controller::player::PlayerSettings;

pub struct PlayerNetworking;

impl Plugin for PlayerNetworking {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (send_players_when_connected, when_disconnected_player));
		app.add_systems(
			Update,
			update_player_position
				.run_if(resource_exists::<MatchboxSocket<MultipleChannels>>),
		);
		app.add_systems(Update, when_spawn_player);
	}
}

fn when_spawn_player(
	mut commands: Commands,
	mut spawn_players: EventReader<SpawnPlayer>,
	asset_server: Res<AssetServer>,
) {
	for player in spawn_players.read() {
	println!("got spawn player");
		let id = spawn_avatar(
			PlayerSettings {
				animations: Some(default_character_animations(&asset_server)),
				vrm: Some(asset_server.load("embedded://models/robot.vrm")),
				void_level: Some(-20.0),
				spawn: Vec3::new(0.0, 3.0, 0.0),
				..default()
			},
			&mut commands,
		);
		commands
			.entity(id)
			.insert(RemotePlayer(player.uuid.clone(), player.peer_id.as_ref().unwrap().clone()));
	}
}

fn when_disconnected_player(
	mut commands: Commands,
	mut event_reader: EventReader<PlayerDisconnected>,
	players: Query<(Entity, &RemotePlayer)>
) {
	for disconnected_peer_id in event_reader.read() {
		for (e, p) in players.iter() {
			if p.1 == disconnected_peer_id.0 {
				commands.entity(e).despawn_recursive();
			}
		}
	}
}

fn send_players_when_connected(
	mut player_connected: EventReader<PlayerConnected>,
	mut connection: Connection,
	local_player: Query<&LocalPlayer>,
) {
	let Ok(local_player) = local_player.get_single() else { return };
	for new_player in player_connected.read() {
		println!("sending connection spawn player");
		let _ = connection.send(
			new_player.0,
			&ReliableMessage::SpawnPlayer(SpawnPlayer {
				uuid: local_player.0.clone(),
				peer_id: None,
			}),
		);
	}
}

fn update_player_position(
	local_player: Query<
		(
			&Position,
			&Rotation,
			&LinearVelocity,
			&AngularVelocity,
			&LocalPlayer,
		),
		(
			Or<(
				Changed<Position>,
				Changed<Rotation>,
				Changed<LinearVelocity>,
			)>,
		),
	>,
	mut connection: Connection,
) {
	let Ok((position, rotation, lin_vel, ang_vel, local_player)) = local_player.get_single() else {return};
	let message = UnreliableMessage::UpdatePhysicsPosition(UpdatePhysicsPosition {
		authority: 0,
		uuid: local_player.0,
		position: position.clone(),
		rotation: rotation.clone(),
		linear_velocity: lin_vel.clone(),
		angular_velocity: ang_vel.clone(),
	});

	let _ = connection.send_all(&message);
}
