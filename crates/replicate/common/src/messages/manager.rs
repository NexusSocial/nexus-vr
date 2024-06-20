use serde::{Deserialize, Serialize};
use url::Url;

use crate::InstanceId;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Serverbound {
	InstanceUrlRequest { id: InstanceId },
	InstanceCreateRequest,
	HandshakeRequest,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Clientbound {
	InstanceUrlResponse { url: Url },
	InstanceCreateResponse { id: InstanceId },
	HandshakeResponse,
}
