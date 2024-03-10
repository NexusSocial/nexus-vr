use std::collections::HashMap;

pub type EntityMap<T> = HashMap<Entity, T>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SpawnedBy {
	Local,
	Remote,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
/// MSB is whether spawning of the entity was initiated remotely or locally.
struct Index(u32);

impl Index {
	fn default_local() -> Self {
		Self(0)
	}

	fn default_remote() -> Self {
		Self(u32::MAX / 2 + 1)
	}

	fn spawned_by(&self) -> SpawnedBy {
		if *self >= Self::default_remote() {
			SpawnedBy::Local
		} else {
			SpawnedBy::Remote
		}
	}
}

/// An identifier for an entity in the network datamodel. NOTE: This is not the
/// same as an ECS entity. This crate is completely independent of bevy.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Entity {
	idx: Index,
}

/// The state of an [`Entity`].
pub type State = bytes::Bytes;

/// Higher values = higher network priority.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Default)]
pub struct Priority(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Changed {
	Deleted,
	UnreliableMutation,
	ReliableMutation,
}

#[derive(Debug, Clone, Default)]
struct EntityData {
	state: State,
	send_prio: Priority,
	recv_prio: Priority,
}

/// Tracks all state of entities.
#[derive(Debug, Clone, Default)]
pub struct DataModel {
	data: EntityMap<EntityData>,
	changes: EntityMap<Changed>,
	local_idx: Index,
}

impl DataModel {
	pub fn new() -> Self {
		Self {
			data: EntityMap::new(),
			changes: EntityMap::new(),
			local_idx: Index::default_local(),
		}
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			data: EntityMap::with_capacity(cap),
			changes: EntityMap::with_capacity(cap),
			local_idx: Index::default_local(),
		}
	}

	/// Reliably updates an entity's state.
	///
	/// "Reliable" here doesn't mean that writes aren't overwritten by other writes,
	/// it just means that the other side of the network is guaranteed to observe it
	/// OR some later write.
	///
	/// This is useful to use when setting a state that you don't expect to change for
	/// a while, to ensure that the other side of the network ends up on the exact
	/// same value.
	///
	/// If you need every single update to be propagated to the other side of the
	/// network, use the events system instead (TODO: make events a thing).
	pub fn update_reliable(
		&mut self,
		entity: Entity,
		state: State,
	) -> Result<(), EntityNotPresent> {
		self.update_inner(entity, state, Changed::ReliableMutation)
	}

	/// Update's an entity's state.
	///
	/// If you need to ensure that the server observes this change, use [`Self::update_reliable`].
	pub fn update(
		&mut self,
		entity: Entity,
		state: State,
	) -> Result<(), EntityNotPresent> {
		self.update_inner(entity, state, Changed::UnreliableMutation)
	}

	fn update_inner(
		&mut self,
		entity: Entity,
		state: State,
		changed: Changed,
	) -> Result<(), EntityNotPresent> {
		let Some(old) = self.data.get_mut(&entity) else {
			return Err(EntityNotPresent);
		};
		old.state = state;
		self.changes
			.insert(entity, changed)
			.expect("already checked presence");
		Ok(())
	}

	/// Spawns an entity, returning its id.
	pub fn spawn(&mut self, state: State) -> Entity {
		let next = Index(self.local_idx.0 + 1);
		if next.spawned_by() != SpawnedBy::Local {
			panic!("ran out of available entities");
		}
		let entity = Entity { idx: next };
		let insert_result = self.data.insert(
			entity,
			EntityData {
				state,
				..Default::default()
			},
		);
		debug_assert!(insert_result.is_none());
		entity
	}

	/// This is not pub, because it should not be called by end users. Only by the client
	/// networking crate.
	fn _spawn_or_update_remote(&mut self, entity: Entity, state: State) {
		assert_eq!(entity.idx.spawned_by(), SpawnedBy::Remote);
		self.data.insert(
			entity,
			EntityData {
				state,
				..Default::default()
			},
		);
	}

	pub fn get(&self, entity: Entity) -> Result<&State, EntityNotPresent> {
		self.data
			.get(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| &data.state)
	}

	pub fn remove(&mut self, entity: Entity) -> Result<(), EntityNotPresent> {
		self.data.remove(&entity).ok_or(EntityNotPresent)?;
		self.changes
			.insert(entity, Changed::Deleted)
			.expect("already checked presence");
		Ok(())
	}

	/// Returns the priorities as `(send, recv)`.
	pub fn priority(
		&mut self,
		entity: Entity,
	) -> Result<(Priority, Priority), EntityNotPresent> {
		self.data
			.get_mut(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| (data.send_prio, data.recv_prio))
	}

	/// Returns mutable refs to the priorities as `(send, recv)`.
	pub fn priority_mut(
		&mut self,
		entity: Entity,
	) -> Result<(&mut Priority, &mut Priority), EntityNotPresent> {
		self.data
			.get_mut(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| (&mut data.send_prio, &mut data.recv_prio))
	}
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("entity not present")]
pub struct EntityNotPresent;

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_dm_new() {
		let mut dm = DataModel::new();

		assert_eq!(dm.local_idx, Index::default_local());

		assert_eq!(dm.data.capacity(), 0);
		assert_eq!(dm.changes.capacity(), 0);

		assert_eq!(dm.get(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.remove(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.priority(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.priority_mut(Entity::default()), Err(EntityNotPresent));
	}

	#[test]
	fn test_dm_capacity() {
		let cap = 10;
		let mut dm = DataModel::with_capacity(cap);

		assert_eq!(dm.local_idx, Index::default_local());

		assert_eq!(dm.data.capacity(), cap);
		assert_eq!(dm.changes.capacity(), cap);

		assert_eq!(dm.get(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.remove(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.priority(Entity::default()), Err(EntityNotPresent));
		assert_eq!(dm.priority_mut(Entity::default()), Err(EntityNotPresent));
	}

	#[test]
	fn test_update_dm() {
		// TODO
	}

	#[test]
	fn test_spawned_by() {
		assert_eq!(Index(u32::MAX).spawned_by(), SpawnedBy::Remote);
		assert_eq!(Index(u32::MAX / 2 + 1).spawned_by(), SpawnedBy::Remote);
		assert_eq!(Index(u32::MAX / 2).spawned_by(), SpawnedBy::Local);
		assert_eq!(Index(u32::MIN).spawned_by(), SpawnedBy::Local);
		assert_eq!(Index(0).spawned_by(), SpawnedBy::Local);
	}

	#[test]
	fn test_spawned_by_defaults() {
		assert_eq!(Index::default_local(), Index(0));
		assert_eq!(2147483648u32, 1u32 << 31);
		assert_eq!(2147483648u32, 1u32 << 31);
		assert_eq!(u32::MAX / 2 + 1, 1u32 << 31);
		assert_eq!(Index::default_remote().0, 1 << 31);
	}
}
