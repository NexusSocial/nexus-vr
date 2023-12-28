//! Lightyear Client that synchronizes the data model with the server.

use std::{
	net::{Ipv4Addr, SocketAddr},
	time::Duration,
};

use bevy::prelude::{Name, Plugin};
use lightyear::prelude::{
	client::{
		Authentication, ClientConfig, InputConfig, InterpolationConfig,
		InterpolationDelay, PluginConfig, PredictionConfig, SyncConfig,
	},
	ClientId, Io, IoConfig, LinkConditionerConfig, PingConfig, TransportConfig,
};

use crate::{
	data_model as dm,
	lightyear::{protocol, shared_config},
	server::{KEY, PROTOCOL_ID},
	Transports,
};

pub const DEFAULT_PORT: u16 = 2234;

pub struct ClientPlugin {
	pub server_addr: SocketAddr,
	pub transport: Transports,
}

impl Plugin for ClientPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		dm::register_types(app);

		let root_entity = app.world.spawn(Name::new("DataModelRoot")).id();
		app.insert_resource(dm::DataModelRoot(root_entity))
			.init_resource::<dm::LocalAvatars>()
			.init_resource::<dm::RemoteAvatars>();

		let client_id: u16 = random_number::random!(); // Larger values overflow
		let client_port = portpicker::pick_unused_port().unwrap_or(DEFAULT_PORT);
		let auth = Authentication::Manual {
			server_addr: self.server_addr,
			client_id: client_id as ClientId,
			private_key: KEY,
			protocol_id: PROTOCOL_ID,
		};
		let client_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), client_port);
		let link_conditioner = LinkConditionerConfig {
			incoming_latency: Duration::from_millis(100),
			incoming_jitter: Duration::from_millis(5),
			incoming_loss: 0.001,
		};
		let transport = match self.transport {
			Transports::Udp => TransportConfig::UdpSocket(client_addr),
		};
		let io = Io::from_config(
			&IoConfig::from_transport(transport).with_conditioner(link_conditioner),
		);
		let config = ClientConfig {
			shared: shared_config().clone(),
			input: InputConfig::default(),
			netcode: Default::default(),
			ping: PingConfig::default(),
			sync: SyncConfig::default(),
			prediction: PredictionConfig::default(),
			// we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
			interpolation: InterpolationConfig::default().with_delay(
				InterpolationDelay::default().with_send_interval_ratio(2.0),
			),
			// .with_delay(InterpolationDelay::Ratio(2.0)),
		};
		let plugin_config = PluginConfig::new(config, io, protocol(), auth);
		app.add_plugins(::lightyear::client::plugin::ClientPlugin::new(
			plugin_config,
		));
	}
}
