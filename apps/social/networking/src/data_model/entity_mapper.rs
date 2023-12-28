//! Data structure for mapping between data model entities and other entities.
#![allow(unused)] // TODO: Maybe we don't need this type after all?

use bevy::{prelude::Entity, reflect::Reflect, utils::HashMap};

/// `Entity` newtype for entities in the data model.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Reflect)]
pub struct DmEntity(pub Entity);

/// Utility for mapping data model entities into other entities
#[derive(Debug, Default, Reflect)]
pub struct EntityMapper {
	// TODO: Switch to EntityHashMap when deriving reflect works.
	dm_to_other: HashMap<DmEntity, Entity>,
	other_to_dm: HashMap<Entity, DmEntity>,
}

impl EntityMapper {
	/// Panics if either entity already existed.
	pub fn insert(&mut self, dm: DmEntity, other: Entity) {
		assert!(
			self.dm_to_other.insert(dm, other).is_none(),
			"`dm` already existed"
		);
		assert!(
			self.other_to_dm.insert(other, dm).is_none(),
			"`other` already existed"
		);
	}

	pub fn dm_to_other(&self) -> &HashMap<DmEntity, Entity> {
		&self.dm_to_other
	}

	pub fn other_to_dm(&self) -> &HashMap<Entity, DmEntity> {
		&self.other_to_dm
	}

	/// Panics if `dm` wasn't present.
	pub fn remove_dm(&mut self, dm: DmEntity) -> Entity {
		let not_dm = self.dm_to_other.remove(&dm).expect("`dm` was not present");
		let removed_dm = self
			.other_to_dm
			.remove(&not_dm)
			.expect("`not_dm` was not present");
		assert_eq!(dm, removed_dm, "sanity check");

		not_dm
	}

	/// Panics if `not_dm` wasn't present.
	pub fn remove(&mut self, not_dm: Entity) -> DmEntity {
		let dm = self
			.other_to_dm
			.remove(&not_dm)
			.expect("`not_dm` was not present");
		let removed_not_dm =
			self.dm_to_other.remove(&dm).expect("`dm` was not present");
		assert_eq!(not_dm, removed_not_dm, "sanity check");
		dm
	}
}
