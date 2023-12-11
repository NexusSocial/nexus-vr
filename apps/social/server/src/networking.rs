use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use social_common::shared::*;
use social_common::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use bevy_vrm::VrmBundle;

pub struct MyServerPlugin {
	pub(crate) port: u16,
	pub(crate) transport: Transports,
}

impl Plugin for MyServerPlugin {
	fn build(&self, app: &mut App) {
		let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.port);
		let netcode_config = NetcodeConfig::default()
			.with_protocol_id(PROTOCOL_ID)
			.with_key(KEY);
		let link_conditioner = LinkConditionerConfig {
			incoming_latency: Duration::from_millis(100),
			incoming_jitter: Duration::from_millis(10),
			incoming_loss: 0.00,
		};
		let transport = match self.transport {
			Transports::Udp => TransportConfig::UdpSocket(server_addr),
			Transports::Webtransport => TransportConfig::WebTransportServer {
				server_addr,
				certificate: Certificate::self_signed(&["localhost"]),
			},
		};
		let io = Io::from_config(
			&IoConfig::from_transport(transport).with_conditioner(link_conditioner),
		);
		let config = ServerConfig {
			shared: shared_config().clone(),
			netcode: netcode_config,
			ping: PingConfig::default(),
		};
		let plugin_config = PluginConfig::new(config, io, protocol());
		app.add_plugins(server::ServerPlugin::new(plugin_config));
		app.add_plugins(shared::SharedPlugin);
		app.init_resource::<Global>();
		app.add_systems(Startup, init);
		// the physics/FixedUpdates systems that consume inputs should be run in this set
		app.add_systems(FixedUpdate, movement.in_set(FixedUpdateSet::Main));
		app.add_systems(Update, (handle_connections, send_message, get_avatar_msg, on_avatar_url_changed, change_pos));
	}
}

#[derive(Resource, Default)]
pub(crate) struct Global {
	pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

pub(crate) fn init(mut commands: Commands) {
	//commands.spawn(Camera2dBundle::default());
	commands.spawn(TextBundle::from_section(
		"Server",
		TextStyle {
			font_size: 30.0,
			color: Color::WHITE,
			..default()
		},
	));
}

/// Server connection system, create a player upon connection
pub(crate) fn handle_connections(
	mut connections: EventReader<ConnectEvent>,
	mut disconnections: EventReader<DisconnectEvent>,
	mut global: ResMut<Global>,
	mut commands: Commands,
) {
	for connection in connections.read() {
		let client_id = connection.context();
		// Generate pseudo random color from client id.
		let h = (((client_id * 30) % 360) as f32) / 360.0;
		let s = 0.8;
		let l = 0.5;
		let entity = commands.spawn(PlayerBundle::new(
			*client_id,
			Vec3::ZERO,
			Color::hsl(h, s, l),
			PlayerAvatarUrl(None),
		));
		// Add a mapping from client id to entity id
		global
			.client_id_to_entity_id
			.insert(*client_id, entity.id());
	}
	for disconnection in disconnections.read() {
		let client_id = disconnection.context();
		if let Some(entity) = global.client_id_to_entity_id.remove(client_id) {
			commands.entity(entity).despawn();
		}
	}
}

/// Read client inputs and move players
pub(crate) fn movement(
	mut position_query: Query<&mut PlayerPosition>,
	mut input_reader: EventReader<InputEvent<Inputs>>,
	global: Res<Global>,
	server: Res<Server<MyProtocol>>,
) {
	for input in input_reader.read() {
		let client_id = input.context();
		if let Some(input) = input.input() {
			debug!(
				"Receiving input: {:?} from client: {:?} on tick: {:?}",
				input,
				client_id,
				server.tick()
			);
			if let Some(player_entity) = global.client_id_to_entity_id.get(client_id) {
				if let Ok(mut position) = position_query.get_mut(*player_entity) {
					shared_movement_behaviour(&mut position, input);
				}
			}
		}
	}
}

/// Send messages from server to clients
pub(crate) fn send_message(
	mut server: ResMut<Server<MyProtocol>>,
	input: Res<Input<KeyCode>>,
) {
	/*if input.pressed(KeyCode::M) {
		// TODO: add way to send message to all
		let message = Message1(5);
		info!("Send message: {:?}", message);
		server
			.broadcast_send::<Channel1, Message1>(Message1(5))
			.unwrap_or_else(|e| {
				error!("Failed to send message: {:?}", e);
			});
	}*/
}

fn get_avatar_msg(
	mut messages: EventReader<MessageEvent<Message1>>,
	mut query: Query<(&PlayerId, &mut PlayerAvatarUrl)>
) {
	for message in messages.read() {
		let id = *message.context();
		let url = message.message().0.clone();
		for (player_id, mut player_avatar_url) in query.iter_mut() {
			if player_id.0 != id {
				continue;
			}
			player_avatar_url.0.replace(url.clone());
		}
	}
}

pub fn change_pos(mut query: Query<(&PlayerPosition, &mut Transform), Changed<PlayerPosition>>) {
	for (player_pos, mut transform) in query.iter_mut() {
		transform.translation = player_pos.0;
	}
}

pub fn on_avatar_url_changed(
	mut commands: Commands,
	assets: Res<AssetServer>,
	mut query: Query<(Entity, &PlayerAvatarUrl), Changed<PlayerAvatarUrl>>,
) {
	for (entity, url) in query.iter() {
		let url = match url.0.as_ref() {
			None => continue,
			Some(url) => url.as_str(),
		};
		let mut transform = Transform::from_xyz(0.0, -1.0, -4.0);
		transform.rotate_y(PI);

		commands.entity(entity).insert(VrmBundle {
			vrm: assets.load(url.to_string()),
			scene_bundle: SceneBundle {
				transform,
				..default()
			},
		});
	}
}