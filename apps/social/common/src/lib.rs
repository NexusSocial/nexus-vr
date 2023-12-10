pub mod shared;

use std::ops::{Add, Mul};
use bevy::prelude::*;
use derive_more::{Add, Mul};
use lightyear::inputs::UserInput;
use lightyear::prelude::*;
use lightyear::protocolize;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transports {
    Udp,
    Webtransport,
}

// Player
#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    position: PlayerPosition,
    color: PlayerColor,
    replicate: Replicate,
}

impl PlayerBundle {
    pub fn new(id: ClientId, position: Vec3, color: Color) -> Self {
        Self {
            id: PlayerId(id),
            position: PlayerPosition(position),
            color: PlayerColor(color),
            replicate: Replicate {
                // prediction_target: NetworkTarget::None,
                prediction_target: NetworkTarget::Only(id),
                interpolation_target: NetworkTarget::AllExcept(id),
                ..default()
            },
        }
    }
}

// Components

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub ClientId);

#[derive(
Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut, Add, Mul)]
pub struct PlayerPosition(pub Vec3);

#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub Color);

// Example of a component that contains an entity.
// This component, when replicated, needs to have the inner entity mapped from the Server world
// to the client World.
// This can be done by adding a `#[message(custom_map)]` attribute to the component, and then
// deriving the `MapEntities` trait for the component.
#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[message(custom_map)]
pub struct PlayerParent(Entity);

impl MapEntities for PlayerParent {
    fn map_entities(&mut self, entity_map: &EntityMap) {
        self.0.map_entities(entity_map);
    }
}

#[component_protocol(protocol = "MyProtocol", derive(Debug))]
pub enum Components {
    #[sync(once)]
    PlayerId(PlayerId),
    #[sync(full)]
    PlayerPosition(PlayerPosition),
    #[sync(once)]
    PlayerColor(PlayerColor),
}

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);

#[message_protocol(protocol = "MyProtocol", derive(Debug))]
pub enum Messages {
    Message1(Message1),
}

// Inputs

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Direction {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Direction {
    pub fn is_none(&self) -> bool {
        !self.up && !self.down && !self.left && !self.right
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Inputs {
    Direction(Direction),
    Delete,
    Spawn,
}

impl UserInput for Inputs {}

// Protocol

protocolize! {
    Self = MyProtocol,
    Message = Messages,
    Component = Components,
    Input = Inputs,
}

pub(crate) fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    });
    protocol
}