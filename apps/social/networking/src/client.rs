//! Lightyear Client that synchronizes the data model with the server.

use std::{
	net::{Ipv4Addr, SocketAddr},
	time::Duration,
};

use bevy::prelude::{
	default, Added, Commands, Entity, Name, Plugin, Query, Res, ResMut, Resource,
	Startup, Update, With,
};
use lightyear::prelude::{
	client::{
		Authentication, ClientConfig, InputConfig, InterpolationConfig,
		InterpolationDelay, PluginConfig, PredictionConfig, SyncConfig,
	},
	ClientId, Io, IoConfig, LinkConditionerConfig, NetworkTarget, PingConfig,
	Replicate, TransportConfig,
};

use crate::data_model::Local;
use crate::lightyear::MyProtocol;
use crate::{
	data_model as dm, data_model,
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
		app.insert_resource(dm::DataModelRoot(root_entity));

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
		app.insert_resource(ClientIdRes(client_id as ClientId));
		app.add_systems(Update, data_model_add_replicated);
		app.add_systems(Startup, connect);
	}
}

#[derive(Resource)]
pub struct ClientIdRes(pub ClientId);
fn data_model_add_replicated(
	mut cmds: Commands,
	added_players: Query<Entity, (Added<data_model::Player>, With<Local>)>,
	client_id: Res<ClientIdRes>,
) {
	for added_player in added_players.iter() {
		cmds.entity(added_player).insert((
			Replicate {
				replication_target: NetworkTarget::All,
				interpolation_target: NetworkTarget::AllExcept(vec![client_id.0]),
				..default()
			},
			data_model::ClientIdComponent(client_id.0),
		));
	}
}

fn connect(mut client: ResMut<lightyear::client::resource::Client<MyProtocol>>) {
	client.connect();
}
