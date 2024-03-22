//! Lightyear Client that synchronizes the data model with the server.

use bevy::app::App;
use std::{
	net::{Ipv4Addr, SocketAddr},
	time::Duration,
};

use bevy::prelude::{
	default, Added, Commands, Entity, Event, EventReader, EventWriter, Name, Plugin,
	Query, Res, Resource, Startup, Update, With,
};
use lightyear::prelude::client::{MessageEvent, NetConfig};
use lightyear::prelude::{
	client::{
		Authentication, ClientConfig, InputConfig, InterpolationConfig,
		InterpolationDelay, PluginConfig, PredictionConfig, SyncConfig,
	},
	ClientId, IoConfig, LinkConditionerConfig, NetworkTarget, PingConfig,
	TransportConfig,
};
use lightyear::shared::replication::components::Replicate;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::data_model::Local;
use crate::lightyear::{MyProtocol, ServerToClientAudioMsg};
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
		let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), client_port);
		let link_conditioner = LinkConditionerConfig {
			incoming_latency: Duration::from_millis(0),
			incoming_jitter: Duration::from_millis(0),
			incoming_loss: 0.00,
		};
		let transport = match self.transport {
			Transports::Udp => TransportConfig::UdpSocket(client_addr),
		};
		let io = IoConfig::from_transport(transport).with_conditioner(link_conditioner);
		let config = ClientConfig {
			shared: shared_config().clone(),
			input: InputConfig::default(),
			ping: PingConfig::default(),
			sync: SyncConfig::default(),
			prediction: PredictionConfig::default(),
			// we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
			interpolation: InterpolationConfig::default().with_delay(
				InterpolationDelay::default(), /*.with_send_interval_ratio(2.0),*/
			),
			net: NetConfig::Netcode {
				auth,
				config: default(),
				io,
			},
			..Default::default() // .with_delay(InterpolationDelay::Ratio(2.0)),
		};
		let plugin_config = PluginConfig::new(config, protocol());
		app.add_plugins(::lightyear::client::plugin::ClientPlugin::new(
			plugin_config,
		));
		app.insert_resource(ClientIdRes(client_id as ClientId));
		app.add_systems(Update, data_model_add_replicated);
		app.add_systems(Startup, connect);
		app.add_plugins(ClientVoiceChat);
	}
}

/// Tracks the client's lightyear ClientId.
#[derive(Resource)]
struct ClientIdRes(ClientId);

fn data_model_add_replicated(
	mut cmds: Commands,
	added_players: Query<Entity, (Added<data_model::Avatar>, With<Local>)>,
	client_id: Res<ClientIdRes>,
) {
	for added_player in added_players.iter() {
		info!("adding replication target for client_id: {:?}", client_id.0);
		cmds.entity(added_player).insert((
			Replicate::<MyProtocol> {
				replication_target: NetworkTarget::All,
				interpolation_target: NetworkTarget::AllExcept(vec![client_id.0]),
				..default()
			},
			data_model::ClientIdComponent(client_id.0),
		));
	}
}

fn connect(mut client: lightyear::client::resource::ClientMut<MyProtocol>) {
	let _ = client.connect();
}

struct ClientVoiceChat;
impl Plugin for ClientVoiceChat {
	fn build(&self, app: &mut App) {
		app.add_event::<ClientToServerVoiceMsg>();
		app.add_event::<ServerToClientVoiceMsg>();
		app.add_systems(Update, send_client_to_server_voice_msg);
		app.add_systems(Update, rec_server_voice_msgs);
	}
}
#[derive(Event)]
pub struct ClientToServerVoiceMsg(pub Vec<u8>, pub Channels);

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Channels(pub u16);

#[derive(Event)]
pub struct ServerToClientVoiceMsg(pub ClientId, pub Vec<u8>, pub Channels);

fn send_client_to_server_voice_msg(
	mut client: lightyear::client::resource::ClientMut<MyProtocol>,
	mut event_reader: EventReader<ClientToServerVoiceMsg>,
) {
	for audio_msg in event_reader.read() {
		client
			.send_message::<crate::lightyear::AudioChannel, _>(
				crate::lightyear::ClientToServerAudioMsg(
					audio_msg.0.clone(),
					audio_msg.1,
				),
			)
			.expect("unable to send message");
	}
}

fn rec_server_voice_msgs(
	mut messages: EventReader<MessageEvent<ServerToClientAudioMsg>>,
	mut event_writer: EventWriter<ServerToClientVoiceMsg>,
) {
	for msg in messages.read() {
		let msg = msg.message();
		let client_id = msg.0;
		let audio = msg.1.clone();
		event_writer.send(ServerToClientVoiceMsg(client_id, audio, msg.2));
	}
}
