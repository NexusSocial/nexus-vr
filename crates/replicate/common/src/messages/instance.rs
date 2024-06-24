use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
	data_model::{
		entity::{Index, Namespace},
		State,
	},
	ClientId,
};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Serverbound {
	HandshakeRequest(ClientId),
	SpawnEntities {
		// Namespace is implied, clients always spawn on their own namespace
		states: HashMap<Index, State>,
	},
	UpdateEntities {
		namespace: Namespace,
		states: HashMap<Index, State>,
	},
	DespawnEntities {
		namespace: Namespace,
		entities: HashSet<Index>,
	},
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum Clientbound {
	HandshakeResponse(Namespace),
	SpawnEntities {
		namespace: Namespace,
		states: HashMap<Index, State>,
	},
	UpdateEntities {
		namespace: Namespace,
		states: HashMap<Index, State>,
	},
	DespawnEntities {
		namespace: Namespace,
		states: HashSet<Index>,
	},
}
