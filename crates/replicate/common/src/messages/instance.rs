use serde::{Deserialize, Serialize};

use crate::data_model::{EntityMap, State};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ServerboundReliable {
	HandshakeRequest,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ClientboundReliable {
	HandshakeResponse,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ServerboundUnreliable {
	Mutations(Mutations),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ClientboundUnreliable {
	Mutations(Mutations),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Mutations(EntityMap<State>);
