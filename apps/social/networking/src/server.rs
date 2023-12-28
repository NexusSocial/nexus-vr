//! Lightyear Client that synchronizes the data model with the server.

use std::{
	net::{Ipv4Addr, SocketAddr},
	time::Duration,
};

use bevy::prelude::{App, Name, Plugin};
use lightyear::{
	prelude::{Io, IoConfig, Key, LinkConditionerConfig, PingConfig, TransportConfig},
	server::{
		config::{NetcodeConfig, ServerConfig},
		plugin::PluginConfig,
	},
};

use crate::{
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
	}
}
