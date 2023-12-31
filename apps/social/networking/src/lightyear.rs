//! The networking data model.

use std::time::Duration;

use lightyear::netcode::ClientId;
use lightyear::{
	prelude::{
		component_protocol, message_protocol, Channel, ChannelDirection, ChannelMode,
		ChannelSettings, LogConfig, ReliableSettings, SharedConfig, TickConfig,
		UserInput,
	},
	protocol::Protocol,
	protocolize,
};
use serde::{Deserialize, Serialize};
use tracing::Level;

use super::data_model;
use data_model::PlayerPoseInterp;

protocolize! {
	Self = MyProtocol,
	Message = Messages,
	Component = Components,
	Input = Inputs,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientToServerAudioMsg(pub Vec<u8>, pub Channels);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerToClientAudioMsg(pub ClientId, pub Vec<u8>, pub Channels);

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
	ClientToServerAudioMsg(ClientToServerAudioMsg),
	ServerToClientAudioMsg(ServerToClientAudioMsg),
}

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
	#[sync(full, lerp = PlayerPoseInterp)]
	PlayerPose(data_model::PlayerPose),
	#[sync(simple)]
	PlayerAvatarUrl(data_model::PlayerAvatarUrl),
	#[sync(simple)]
	Player(data_model::Avatar),
	#[sync(simple)]
	ClientIdComponent(data_model::ClientIdComponent),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Inputs {}

impl UserInput for Inputs {}

// Channels

#[derive(Channel)]
pub struct Channel1;

#[derive(Channel)]
pub struct AudioChannel;

pub fn protocol() -> MyProtocol {
	let mut protocol = MyProtocol::default();
	protocol.add_channel::<Channel1>(ChannelSettings {
		mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
		direction: ChannelDirection::Bidirectional,
	});
	protocol.add_channel::<AudioChannel>(ChannelSettings {
		mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
		direction: ChannelDirection::Bidirectional,
	});
	protocol
}

pub fn shared_config() -> SharedConfig {
	SharedConfig {
        enable_replication: true,
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(100),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        log: LogConfig {
            level: Level::INFO,
            filter: "wgpu=error,wgpu_hal=error,naga=warn,bevy_app=info,bevy_render=warn,quinn=warn"
                .to_string(),
        },
    }
}

pub use lightyear::prelude::client::Interpolated;
use lightyear::prelude::Message;
use crate::client::Channels;
