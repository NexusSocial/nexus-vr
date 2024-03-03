use serde::{Deserialize, Serialize};

use crate::InstanceId;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Serverbound {
	InstanceCreateRequest,
	HandshakeRequest,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Clientbound {
	InstanceCreateResponse { id: InstanceId },
	HandshakeResponse,
}
