mod entity_map;

use std::num::NonZeroU16;

use bytes::Bytes;

use self::entity_map::EntityMap;

/// The state of an [`Entity`].
/// Guarantees that `Bytes` must not be empty. This allows for more efficient
/// occupancy checks in the [`EntityMap`].
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct State(bytes::Bytes);

impl State {
	pub fn get(&self) -> &Bytes {
		&self.0
	}
}

impl TryFrom<Bytes> for State {
	type Error = Bytes;

	fn try_from(value: Bytes) -> Result<Self, Self::Error> {
		if value.is_empty() {
			Err(value)
		} else {
			Ok(Self(value))
		}
	}
}

/// An identifier for an entity in the network datamodel. NOTE: This is not the
/// same as an ECS entity. This crate is completely independent of bevy.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Entity(NonZeroU16);

/// Elements of
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataModel {
	states: EntityMap<State>,
}
