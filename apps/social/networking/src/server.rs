//! Lightyear Client that synchronizes the data model with the server.

use bevy::app::PreUpdate;
use std::{
	net::{Ipv4Addr, SocketAddr},
	time::Duration,
};

use bevy::prelude::{default, Added, App, Commands, Entity, EventReader, Name, Plugin, Query, ResMut, Update, IntoSystemConfigs};
use lightyear::prelude::server::MessageEvent;
use lightyear::prelude::{NetworkTarget, Replicate};
use lightyear::{
	prelude::{Io, IoConfig, Key, LinkConditionerConfig, PingConfig, TransportConfig},
	server::{
		config::{NetcodeConfig, ServerConfig},
		plugin::PluginConfig,
	},
};
use lightyear::prelude::MainSet::ClientReplication;
use lightyear::server::resource::Server;
use tracing::info;

use crate::data_model::ClientIdComponent;
use crate::lightyear::{
	AudioChannel, ClientToServerAudioMsg, MyProtocol, ServerToClientAudioMsg,
};
use crate::{
	data_model,
	data_model::{register_types, DataModelRoot},
	lightyear::{protocol, shared_config},
	Transports,
};

pub const PROTOCOL_ID: u64 = 0;
pub const KEY: Key = [0; 32];
pub const DEFAULT_PORT: u16 = 5000;

pub struct ServerPlugin {
	pub port: u16,
	pub transport: Transports,
}

impl Default for ServerPlugin {
	fn default() -> Self {
		Self {
			port: DEFAULT_PORT,
			transport: Transports::Udp,
		}
	}
}

impl Plugin for ServerPlugin {
	fn build(&self, app: &mut App) {
		register_types(app);
		let root_entity = app.world.spawn(Name::new("DataModelRoot")).id();
		app.insert_resource(DataModelRoot(root_entity));

		let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), self.port);
		let netcode_config = NetcodeConfig::default()
			.with_protocol_id(PROTOCOL_ID)
			.with_key(KEY);
		let link_conditioner = LinkConditionerConfig {
			incoming_latency: Duration::from_millis(0),
			incoming_jitter: Duration::from_millis(0),
			incoming_loss: 0.00,
		};
		let transport = match self.transport {
			Transports::Udp => TransportConfig::UdpSocket(server_addr),
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
		app.add_plugins(::lightyear::server::plugin::ServerPlugin::new(
			plugin_config,
		));
		app.add_systems(PreUpdate, add_replication_for_players.in_set(ClientReplication));
		app.add_plugins(ServerVoiceChat);
	}
}

fn add_replication_for_players(
	mut cmds: Commands,
	added_player: Query<(Entity, &ClientIdComponent), Added<data_model::Avatar>>,
	server: ResMut<Server<MyProtocol>>,
) {
	//info!("server client ids: {:#?}", server.client_ids().collect::<Vec<_>>());
	for (entity, client_id) in added_player.iter() {
		info!("replicate client id: {:?}", client_id);
		cmds.entity(entity).insert(Replicate {
			replication_target: NetworkTarget::None,
			// we want the other clients to apply interpolation for the cursor
			interpolation_target: NetworkTarget::None,
			..default()
		});
	}
}

pub struct ServerVoiceChat;
impl Plugin for ServerVoiceChat {
	fn build(&self, app: &mut App) {
		app.add_systems(PreUpdate, re_broadcast_audio);
	}
}

fn re_broadcast_audio(
	mut messages: EventReader<MessageEvent<ClientToServerAudioMsg>>,
	mut server: ResMut<lightyear::server::resource::Server<MyProtocol>>,
) {
	for message in messages.read() {
		let id2 = *message.context();
		let audio = message.message().clone().0;
		let channels = message.message().1;
		for id in server.client_ids().collect::<Vec<_>>() {
			if id == id2 {
				continue;
			}
			server
				.send_message::<AudioChannel, _>(
					id,
					ServerToClientAudioMsg(id2, audio.clone(), channels),
				)
				.unwrap();
		}
	}
}
