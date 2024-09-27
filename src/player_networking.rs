use crate::networking::{Connection, ConnectionTrait, LocalPlayer, PlayerConnected, PlayerDisconnected, PlayerPositionUpdate, ReliableMessage, RemotePlayer, SpawnPlayer, UnreliableMessage, UpdatePhysicsPosition};
use crate::spawn_avatar;
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use bevy_matchbox::prelude::MultipleChannels;
use bevy_matchbox::MatchboxSocket;
use bevy_mod_openxr::session::OxrSession;
use bevy_vr_controller::animation::defaults::default_character_animations;
use bevy_vr_controller::player::PlayerSettings;
use bevy_vrm::BoneName;

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
			Entity,
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
	transforms: Query<(&BoneName, &Transform), Changed<Transform>>,
	children: Query<&Children>,
	session: Option<Res<OxrSession>>,
) {
	let Ok((entity, position, rotation, lin_vel, ang_vel, local_player)) = local_player.get_single() else {return};
	let message = UnreliableMessage::UpdatePhysicsPosition(UpdatePhysicsPosition {
		authority: 0,
		uuid: local_player.0,
		position: position.clone(),
		rotation: rotation.clone(),
		linear_velocity: lin_vel.clone(),
		angular_velocity: ang_vel.clone(),
	});

	let _ = connection.send_all(&message);

	if session.is_none() {
		return;
	}

	let mut player_position_update = PlayerPositionUpdate {
		uuid: local_player.0.clone(),
		head: Default::default(),
		left_shoulder: Default::default(),
		right_shoulder: Default::default(),
		left_upper_arm: Default::default(),
		right_upper_arm: Default::default(),
		left_lower_arm: Default::default(),
		right_lower_arm: Default::default(),
		left_hand: Default::default(),
		right_hand: Default::default(),
	};
	for children in children.iter_descendants(entity) {
		for (bone_name, transform) in transforms.get(children) {
			match bone_name {
				BoneName::Head => player_position_update.head = transform.clone(),
				BoneName::LeftShoulder => player_position_update.left_shoulder = transform.clone(),
				BoneName::RightShoulder => player_position_update.right_shoulder = transform.clone(),
				BoneName::LeftUpperArm => player_position_update.left_upper_arm = transform.clone(),
				BoneName::RightUpperArm => player_position_update.right_upper_arm = transform.clone(),
				BoneName::LeftLowerArm => player_position_update.left_lower_arm = transform.clone(),
				BoneName::RightLowerArm => player_position_update.right_lower_arm = transform.clone(),
				BoneName::LeftHand => player_position_update.left_hand = transform.clone(),
				BoneName::RightHand => player_position_update.right_hand = transform.clone(),
				_ => {}
			}
		}
	}
	let _ = connection.send_all(&UnreliableMessage::PlayerPositionUpdate(player_position_update));
}
