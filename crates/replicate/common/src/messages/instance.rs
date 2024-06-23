use serde::{Deserialize, Serialize};

use crate::data_model::entity::{EntityId, Index, Namespace};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Serverbound {
	HandshakeRequest,
	SpawnEntities { idxs: Vec<Index> },
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Clientbound {
	HandshakeResponse(Namespace),
	SpawnEntities { ids: Vec<EntityId> },
}
