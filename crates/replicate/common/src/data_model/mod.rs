use std::collections::HashMap;

pub type EntityMap<T> = HashMap<Entity, T>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SpawnedBy {
	Local,
	Remote,
}

/// MSB is whether spawning of the entity was initiated remotely or locally.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
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
			SpawnedBy::Remote
		} else {
			SpawnedBy::Local
		}
	}

	/// The next entity's index.
	///
	/// # Panics
	/// Panics if the number of entities overflows.
	fn next(&self) -> Self {
		let before = self.spawned_by();
		let result = Self(self.0.wrapping_add(1));
		assert_eq!(result.spawned_by(), before, "ran out of available entities");

		result
	}
}

/// An identifier for an entity in the network datamodel. NOTE: This is not the
/// same as an ECS entity. This crate is completely independent of bevy.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity {
	// TODO: Maybe there should be no difference between the index and the entity?
	idx: Index,
}

impl Entity {
	pub fn default_local() -> Self {
		Self {
			idx: Index::default_local(),
		}
	}

	pub fn default_remote() -> Self {
		Self {
			idx: Index::default_remote(),
		}
	}
}

/// The state of an [`Entity`].
pub type State = bytes::Bytes;

/// Higher values = higher network priority.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Default)]
pub struct Priority(pub u8);

/// The type of state mutation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StateMutation {
	/// The state has been updated. Subsequent mutations may overwrite this value.
	Unreliable,
	/// The state has been updated. Subsequent mutations may overwrite this value. It is
	/// guaranteed that the remote peer will observe this or a later mutation.
	Reliable,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct EntityData {
	state: State,
	send_prio: Priority,
	recv_prio: Priority,
}

/// The local changes for entities since the last network flush.
///
/// Intended to be sent to a separate network task and written over the network to the
/// remote peer.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct LocalChanges {
	/// The state when spawning is always guaranteed to be observed by the remote peer.
	pub spawns: EntityMap<State>,
	/// The state at the time of despawn is always guaranteed to be observed by the
	/// remote peer.
	pub despawns: EntityMap<State>,
	/// Only the latest state at the time of network sync will be sent to the remote
	/// peer, and whether it is sent reliably or unreliably depends on whether a
	/// reliable mutation is pending.
	pub mutations: EntityMap<(StateMutation, State)>,
}

/// The *pending* version of [`LocalChanges`]. These are built up internally as the
/// local data model is mutated. At any time, the data model can be flushed and
/// [`PendingLocalChanges`] are converted to [`LocalChanges`].
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct PendingLocalChanges {
	spawns: EntityMap<State>,
	despawns: EntityMap<State>,
	mutations: EntityMap<StateMutation>,
}

impl PendingLocalChanges {
	#[cfg(test)]
	fn is_empty(&self) -> bool {
		self.spawns.is_empty() && self.despawns.is_empty() && self.mutations.is_empty()
	}
}

/// Tracks the remote changes for an entity from the last network sync. Allows building
/// up these changes in a network task, and applying them all at once when the data
/// model is unlocked.
///
/// These changes are intended to be added to in a networking task, and then
/// applied just before game logic is scheduled to tick.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RemoteChanges {
	/// Spawns done by remote peers. Applied first.
	spawns: EntityMap<EntityData>,
	/// Despawns done by remote peers. Applied last.
	despawns: EntityMap<State>,
	/// Updates. Applied after spawns.
	updates: EntityMap<EntityData>,
}

/// Tracks all state of entities.
#[derive(Debug, Clone)]
pub struct DataModel {
	data: EntityMap<EntityData>,
	/// Tracks local changes that haven't been flushed yet.
	pending: PendingLocalChanges,
	/// The last index of the locally spawned entities.
	local_idx: Index,
}

impl DataModel {
	pub fn new() -> Self {
		Self {
			data: EntityMap::new(),
			pending: PendingLocalChanges::default(),
			local_idx: Index::default_local(),
		}
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			data: EntityMap::with_capacity(cap),
			pending: PendingLocalChanges::default(),
			local_idx: Index::default_local(),
		}
	}

	/// Spawns an entity, returning its id.
	///
	/// # Panics
	/// Panics if the number of entities overflows.
	pub fn spawn(&mut self, state: State) -> Entity {
		let next = self.local_idx.next();
		let entity = Entity { idx: next };
		let insert_result = self.data.insert(
			entity,
			EntityData {
				state: state.clone(),
				..Default::default()
			},
		);
		debug_assert!(
			insert_result.is_none(),
			"sanity: can't spawn on top of existing entities"
		);

		let insert_result = self.pending.spawns.insert(entity, state);
		debug_assert!(
			insert_result.is_none(),
			"sanity: can't spawn same entity twice"
		);

		self.local_idx = next;
		entity
	}

	pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityNotPresent> {
		let deleted_data = self.data.remove(&entity).ok_or(EntityNotPresent)?;
		let insert_result = self.pending.despawns.insert(entity, deleted_data.state);
		debug_assert!(insert_result.is_none(), "sanity: can't despawn twice");

		Ok(())
	}

	/// Updates an entity's state.
	///
	/// When the changes are flushed to the network, only the *latest* state rather than
	/// all intermediate states are sent to the remote peer. This is to ensure the best
	/// possible latency for a state change, as the fundamental assumption of states is
	/// that the latest value (other than the value at (de)spawn) is the only one that
	/// matters.
	///
	/// This makes `update` unsuitable for sending events where there needs to be a
	/// guarantee that every event is observed by the remote peer. For that use case,
	/// use the events system instead (TODO: make events a thing).
	///
	/// However, if you only want to guarantee that the *latest* value is observed by
	/// the remote peer, you can use [`Self::update_reliable`] which is more efficient
	/// than the events system when you only care about the latest value being observed.
	pub fn update(
		&mut self,
		entity: Entity,
		state: State,
	) -> Result<(), EntityNotPresent> {
		self.update_inner(entity, state, StateMutation::Unreliable)
	}

	/// Reliably updates an entity's state. Almost always, you should use
	/// [`Self::update`] instead as it is lower latency.
	///
	/// "Reliable" means that on the next network flush, the *latest* state will be sent
	/// with reliable networking, ensuring that the remote peer observes the change.
	///
	/// This is useful to use when setting a state that you don't expect to change for a
	/// while, to ensure that the remote peer ends up on the exact same value.
	///
	/// When the changes are flushed to the network, only the *latest* state rather than
	/// all intermediate states are sent to the remote peer. This is to ensure the best
	/// possible latency for a state change, as the fundamental assumption of states is
	/// that the latest value (other than the value at (de)spawn) is the only one that
	/// matters.
	///
	/// This makes `update_reliable` unsuitable for sending events where there needs to
	/// be a guarantee that every event is observed by the remote peer. For that use
	/// case, use the events system instead (TODO: make events a thing).
	pub fn update_reliable(
		&mut self,
		entity: Entity,
		state: State,
	) -> Result<(), EntityNotPresent> {
		self.update_inner(entity, state, StateMutation::Reliable)
	}

	fn update_inner(
		&mut self,
		entity: Entity,
		state: State,
		mutation: StateMutation,
	) -> Result<(), EntityNotPresent> {
		// update main data model
		let Some(old) = self.data.get_mut(&entity) else {
			return Err(EntityNotPresent);
		};
		old.state = state;

		// update pending changes.
		let entry = self
			.pending
			.mutations
			.entry(entity)
			.or_insert(StateMutation::Unreliable);
		if mutation == StateMutation::Reliable {
			// reliable mutation will set the change to reliable even if it was
			// previously set to unreliable.
			*entry = StateMutation::Reliable;
		}

		Ok(())
	}

	pub fn get(&self, entity: Entity) -> Result<&State, EntityNotPresent> {
		self.data
			.get(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| &data.state)
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

	/// Applies a set of pending remote changes to the data model, and returns the set
	/// of local changes since the last flush that the network task should use.
	///
	/// This allows the network task to avoid needing to lock the data model, and
	/// instead can simply reference the contents of `LocalChanges`, as well as build up
	/// the changes in `RemoteChanges`.
	///
	/// Note that the `local_changes` are taken by mutable ref, to allow reusing
	/// allocations. The caller should read the mutated `local_changes` to see the
	/// result of the function.
	pub fn flush(
		&mut self,
		_remote_changes: &RemoteChanges,
		_local_changes: &mut LocalChanges,
	) {
		todo!()
	}
}

impl Default for DataModel {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("entity not present")]
pub struct EntityNotPresent;

#[cfg(test)]
mod test_pending {
	use super::*;

	#[test]
	fn test_pending_local_is_empty() {
		assert!(PendingLocalChanges::default().is_empty());
		let empty_state = bytes::Bytes::new();

		assert!(!PendingLocalChanges {
			spawns: EntityMap::from([(Entity::default_local(), empty_state.clone())]),
			..Default::default()
		}
		.is_empty());
		assert!(!PendingLocalChanges {
			despawns: EntityMap::from([(Entity::default_local(), empty_state.clone())]),
			..Default::default()
		}
		.is_empty());
		assert!(!PendingLocalChanges {
			mutations: EntityMap::from([(
				Entity::default_local(),
				StateMutation::Unreliable
			)]),
			..Default::default()
		}
		.is_empty());
	}
}

#[cfg(test)]
mod test_dm {
	use super::*;

	#[test]
	fn test_new() {
		let mut dm = DataModel::new();

		assert_eq!(dm.local_idx, Index::default_local());
		assert_eq!(dm.data.capacity(), 0);
		assert!(dm.pending.is_empty());

		assert_eq!(dm.get(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(dm.despawn(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(dm.priority(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(
			dm.priority_mut(Entity::default_local()),
			Err(EntityNotPresent)
		);
	}

	#[test]
	fn test_capacity() {
		let cap = 10;
		let mut dm = DataModel::with_capacity(cap);

		assert_eq!(dm.local_idx, Index::default_local());
		assert!(dm.data.capacity() >= cap);
		assert!(dm.pending.is_empty());

		assert_eq!(dm.get(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(dm.despawn(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(dm.priority(Entity::default_local()), Err(EntityNotPresent));
		assert_eq!(
			dm.priority_mut(Entity::default_local()),
			Err(EntityNotPresent)
		);
	}

	#[test]
	fn test_spawn() {
		let mut dm = DataModel::new();
		// Sanity checks
		assert_eq!(dm.local_idx, Index::default_local());
		assert!(dm.pending.is_empty());

		// Spawn the first entity
		let expected_state1 = bytes::Bytes::from_static(&[1]);
		let expected_index1 = Index::default_local().next();
		let entity1 = dm.spawn(expected_state1.clone());

		fn check_entity(
			dm: &DataModel,
			entity: Entity,
			expected_state: &State,
			expected_idx: Index,
		) {
			// Check expected values of entity
			assert_eq!(entity.idx, expected_idx);
			// Check expected values of entity's state.
			assert_eq!(dm.get(entity), Ok(expected_state));
			// Check expected values of change detection
			assert_eq!(dm.pending.spawns[&entity], expected_state.clone());
			assert!(!dm.pending.despawns.contains_key(&entity));
			assert!(!dm.pending.mutations.contains_key(&entity));
		}

		check_entity(&dm, entity1, &expected_state1, expected_index1);
		// Check expected value of local_idx
		assert_eq!(dm.local_idx, entity1.idx);
		// Check expected values of change detection
		assert!(!dm.pending.is_empty());
		assert_eq!(dm.pending.spawns.len(), 1);
		assert_eq!(dm.pending.despawns.len(), 0);
		assert_eq!(dm.pending.mutations.len(), 0);

		// Spawn another entity
		let expected_state2 = bytes::Bytes::from_static(&[2]);
		let expected_index2 = Index::default_local().next().next();
		let entity2 = dm.spawn(expected_state2.clone());

		check_entity(&dm, entity1, &expected_state1, expected_index1);
		check_entity(&dm, entity2, &expected_state2, expected_index2);
		// Check expected value of local_idx
		assert_eq!(dm.local_idx, entity2.idx);
		// Check expected values of change detection
		assert!(!dm.pending.is_empty());
		assert_eq!(dm.pending.spawns.len(), 2);
		assert_eq!(dm.pending.despawns.len(), 0);
		assert_eq!(dm.pending.mutations.len(), 0);
	}
}

#[cfg(test)]
mod test_index {
	use super::*;

	fn max_local() -> Index {
		let idx = Index(u32::MAX / 2);
		assert_eq!(idx.spawned_by(), SpawnedBy::Local);

		let higher = Index(idx.0.wrapping_add(1));
		assert_eq!(higher.spawned_by(), SpawnedBy::Remote);

		let lower = Index(idx.0.wrapping_sub(1));
		assert_eq!(lower.spawned_by(), SpawnedBy::Local);

		idx
	}

	fn min_local() -> Index {
		let idx = Index(0);
		assert_eq!(idx.spawned_by(), SpawnedBy::Local);

		let higher = Index(idx.0.wrapping_add(1));
		assert_eq!(higher.spawned_by(), SpawnedBy::Local);

		let lower = Index(idx.0.wrapping_sub(1));
		assert_eq!(lower.spawned_by(), SpawnedBy::Remote);

		idx
	}

	fn max_remote() -> Index {
		let idx = Index(u32::MAX);
		assert_eq!(idx.spawned_by(), SpawnedBy::Remote);

		let higher = Index(idx.0.wrapping_add(1));
		assert_eq!(higher.spawned_by(), SpawnedBy::Local);

		let lower = Index(idx.0.wrapping_sub(1));
		assert_eq!(lower.spawned_by(), SpawnedBy::Remote);

		idx
	}

	fn min_remote() -> Index {
		let idx = Index(u32::MAX / 2 + 1);
		assert_eq!(idx.spawned_by(), SpawnedBy::Remote);

		let higher = Index(idx.0.wrapping_add(1));
		assert_eq!(higher.spawned_by(), SpawnedBy::Remote);

		let lower = Index(idx.0.wrapping_sub(1));
		assert_eq!(lower.spawned_by(), SpawnedBy::Local);

		idx
	}

	#[test]
	fn test_next() {
		let default_local = Index::default_local();
		assert_eq!(default_local.0, 0);
		assert_eq!(default_local.next(), Index(1));

		let default_remote = Index::default_remote();
		assert_eq!(default_remote.0, u32::MAX / 2 + 1);
		assert_eq!(default_remote.next(), Index(u32::MAX / 2 + 2));
	}

	#[test]
	#[should_panic(expected = "ran out of available entities")]
	fn test_next_max_remote_overflows() {
		max_remote().next();
	}

	#[test]
	#[should_panic(expected = "ran out of available entities")]
	fn test_next_max_local_overflows() {
		max_local().next();
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
		assert_eq!(Index::default_local(), min_local());
		assert_eq!(Index::default_remote(), min_remote());
	}
}
