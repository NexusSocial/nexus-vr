use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Serverbound {
	HandshakeRequest,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Clientbound {
	HandshakeResponse,
}
