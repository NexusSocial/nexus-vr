use serde::{Deserialize, Serialize};

/// The canonical representation of an entity id.
///
/// Entities area represented as an index along with a random namespace.
///
/// NOTE: This is not the same as an entity in (insert game engine here). This crate
/// is completely independent from your game engine.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct EntityId {
	pub namespace: Namespace,
	pub idx: Index,
}

/// Together with [`Namespace`], this makes up an [`EntityId`].
///
/// Clients may choose an Index however they want, and the server will accept it.
/// Typically, the number is just incremented every time an entity is spawned.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub struct Index(pub u32);

impl Index {
	/// The next index.
	pub fn next(&self) -> Index {
		Self(
			self.0
				.checked_add(1)
				.expect("ran out of available entities"),
		)
	}
}

/// Together with [`Index`], this makes up an [`EntityId`].
/// Namespaces are universally unique. This allows clients to generate new `EntityId`s
/// without colliding with existing ones.
///
/// The server has authority over the namespace that a client is supposed to use.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Namespace(pub u64);
