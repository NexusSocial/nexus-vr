mod player_pose;

pub(crate) use self::player_pose::PlayerPoseInterp;
pub use self::player_pose::{Isometry, PlayerPose};

use bevy::{
	prelude::{App, Component, Entity, Resource},
	reflect::Reflect,
	utils::HashMap,
};
use lightyear::prelude::{client::InterpolatedComponent, Message};
use serde::{Deserialize, Serialize};

/// `Entity` newtype for entities in the data model.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Reflect)]
pub struct DmEntity(pub Entity);

/// Utility for mapping data model entities into other entities
#[derive(Debug, Default, Reflect)]
pub struct EntityMapper {
	// TODO: Switch to EntityHashMap when deriving reflect works.
	from_dm: HashMap<DmEntity, Entity>,
	to_dm: HashMap<Entity, DmEntity>,
}

impl EntityMapper {
	/// Panics if either entity already existed.
	pub fn insert(&mut self, dm: DmEntity, other: Entity) {
		assert!(
			self.from_dm.insert(dm, other).is_none(),
			"`dm` already existed"
		);
		assert!(
			self.to_dm.insert(other, dm).is_none(),
			"`other` already existed"
		);
	}

	pub fn from_dm(&self) -> &HashMap<DmEntity, Entity> {
		&self.from_dm
	}

	pub fn to_dm(&self) -> &HashMap<Entity, DmEntity> {
		&self.to_dm
	}

	/// Panics if `dm` wasn't present.
	pub fn remove_dm(&mut self, dm: DmEntity) -> Entity {
		let not_dm = self.from_dm.remove(&dm).expect("`dm` was not present");
		let removed_dm = self
			.to_dm
			.remove(&not_dm)
			.expect("`not_dm` was not present");
		assert_eq!(dm, removed_dm, "sanity check");

		not_dm
	}

	/// Panics if `not_dm` wasn't present.
	pub fn remove(&mut self, not_dm: Entity) -> DmEntity {
		let dm = self
			.to_dm
			.remove(&not_dm)
			.expect("`not_dm` was not present");
		let removed_not_dm = self.from_dm.remove(&dm).expect("`dm` was not present");
		assert_eq!(not_dm, removed_not_dm, "sanity check");
		dm
	}
}

/// Will read from these into the data model.
#[derive(Resource, Reflect, Default, Debug)]
pub struct LocalAvatars(pub EntityMapper);

/// Will read from the data model into these.
#[derive(Resource, Reflect, Default, Debug)]
pub struct RemoteAvatars(pub EntityMapper);

/// All data in the data model has this entity as its ancestor.
#[derive(Resource, Reflect, Debug)]
pub struct DataModelRoot(pub Entity);

#[derive(
	Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect,
)]
pub struct PlayerAvatarUrl(pub Option<String>);

pub(crate) fn register_types(app: &mut App) {
	app.register_type::<DataModelRoot>()
		.register_type::<LocalAvatars>()
		.register_type::<RemoteAvatars>()
		.register_type::<PlayerAvatarUrl>()
		.register_type::<RemoteAvatars>();
}

// ---- type checks ----

fn _assert_impl_interp() {
	fn check<T: InterpolatedComponent<T>>() {}

	check::<PlayerPose>()
}
