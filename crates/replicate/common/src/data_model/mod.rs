use std::collections::HashMap;

pub mod entity;
use self::entity::{EntityId, Index, Namespace};

pub type EntityMap<T> = HashMap<EntityId, T>;

/// The state of an entity.
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
	// TODO: This could be Index -> State
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
///
/// The reason that we have this type in addition to [`LocalChanges`] is because
/// [`PendingLocalChanges`] refers to the states in the data model and therefore to
/// read the value of a mutated entity's state requires access to [`DataModel::data`],
/// whereas [`LocalChanges`] is self-contained and doesn't require any access to the
/// [`DataModel`]. It is important for [`LocalChanges`] to be self-contained because it
/// is going to be sent to a separate task and won't necessarily have read access
/// anymore to the overall data model.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct PendingLocalChanges {
	// TODO: This could be Index -> State
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
	/// Updates. Applied after spawns.
	updates: EntityMap<EntityData>,
	/// Despawns done by remote peers. Applied last.
	despawns: EntityMap<State>,
}

/// Tracks all state of entities.
#[derive(Debug, Clone)]
pub struct DataModel {
	data: EntityMap<EntityData>,
	/// Tracks local changes that haven't been flushed yet.
	pending: PendingLocalChanges,
	/// The next free index for locally spawned entities.
	local_free_idx: Index,
	local_namespace: Namespace,
}

impl DataModel {
	/// Creates a `DataModel`.
	///
	/// Any entities spawned with [`Self::spawn`] will use `local_namespace` as their
	/// [`Namespace`].
	pub fn new(local_namespace: Namespace) -> Self {
		Self {
			data: EntityMap::new(),
			pending: PendingLocalChanges::default(),
			local_free_idx: Index::default(),
			local_namespace,
		}
	}

	pub fn with_capacity(local_namespace: Namespace, cap: usize) -> Self {
		Self {
			data: EntityMap::with_capacity(cap),
			pending: PendingLocalChanges::default(),
			local_free_idx: Index::default(),
			local_namespace,
		}
	}

	/// Any entities spawned locally will have this namespace.
	pub fn local_namespace(&self) -> Namespace {
		self.local_namespace
	}

	/// Spawns an entity, returning its id.
	///
	/// # Panics
	/// Panics if the number of entities overflows.
	pub fn spawn(&mut self, state: State) -> EntityId {
		let entity = EntityId {
			idx: self.local_free_idx,
			namespace: self.local_namespace,
		};
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

		self.local_free_idx = self.local_free_idx.next();
		entity
	}

	pub fn despawn(&mut self, entity: EntityId) -> Result<(), EntityNotPresent> {
		let deleted_data = self.data.remove(&entity).ok_or(EntityNotPresent)?;
		let insert_result = self.pending.despawns.insert(entity, deleted_data.state);
		debug_assert!(insert_result.is_none(), "sanity: can't despawn twice");
		// No point sending mutations when we already send despawn state
		self.pending.mutations.remove(&entity);

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
		entity: EntityId,
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
		entity: EntityId,
		state: State,
	) -> Result<(), EntityNotPresent> {
		self.update_inner(entity, state, StateMutation::Reliable)
	}

	fn update_inner(
		&mut self,
		entity: EntityId,
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

	pub fn get(&self, entity: EntityId) -> Result<&State, EntityNotPresent> {
		self.data
			.get(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| &data.state)
	}

	/// Returns the priorities as `(send, recv)`.
	pub fn priority(
		&mut self,
		entity: EntityId,
	) -> Result<(Priority, Priority), EntityNotPresent> {
		self.data
			.get_mut(&entity)
			.ok_or(EntityNotPresent)
			.map(|data| (data.send_prio, data.recv_prio))
	}

	/// Returns mutable refs to the priorities as `(send, recv)`.
	pub fn priority_mut(
		&mut self,
		entity: EntityId,
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
		local_changes: &mut LocalChanges,
	) {
		// TODO: For now, we aren't actually applying any remote changes, because the
		// server isn't implemented.

		local_changes.mutations.clear();
		local_changes.despawns.clear();
		local_changes.mutations.clear();

		local_changes.spawns.extend(self.pending.spawns.drain());
		local_changes.despawns.extend(self.pending.despawns.drain());
		local_changes
			.mutations
			.extend(self.pending.mutations.iter().map(|(&entity, &mutation)| {
				let state = self.get(entity).expect("entity should be present").clone();
				(entity, (mutation, state))
			}));
		self.pending.mutations.clear();
	}
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("entity not present")]
pub struct EntityNotPresent;

#[cfg(test)]
mod test_pending {
	use super::*;

	const TEST_NAMESPACE: Namespace = Namespace(1337);

	fn local_id(idx: u32) -> EntityId {
		EntityId {
			idx: Index(idx),
			namespace: TEST_NAMESPACE,
		}
	}

	#[test]
	fn test_pending_local_is_empty() {
		assert!(PendingLocalChanges::default().is_empty());
		let empty_state = bytes::Bytes::new();

		assert!(!PendingLocalChanges {
			spawns: EntityMap::from([(local_id(0), empty_state.clone())]),
			..Default::default()
		}
		.is_empty());
		assert!(!PendingLocalChanges {
			despawns: EntityMap::from([(local_id(0), empty_state.clone())]),
			..Default::default()
		}
		.is_empty());
		assert!(!PendingLocalChanges {
			mutations: EntityMap::from([(local_id(0), StateMutation::Unreliable)]),
			..Default::default()
		}
		.is_empty());
	}
}

#[cfg(test)]
mod test_dm {
	use super::*;

	const TEST_NAMESPACE: Namespace = Namespace(1337);

	fn local_id(idx: u32) -> EntityId {
		EntityId {
			idx: Index(idx),
			namespace: TEST_NAMESPACE,
		}
	}

	#[test]
	fn test_new() {
		let mut dm = DataModel::new(TEST_NAMESPACE);

		assert_eq!(dm.local_free_idx, Index::default());
		assert_eq!(dm.data.capacity(), 0);
		assert!(dm.pending.is_empty());

		assert_eq!(dm.get(local_id(0)), Err(EntityNotPresent));
		assert_eq!(dm.despawn(local_id(0)), Err(EntityNotPresent));
		assert_eq!(dm.priority(local_id(0)), Err(EntityNotPresent));
		assert_eq!(dm.priority_mut(local_id(0)), Err(EntityNotPresent));
	}

	#[test]
	fn test_capacity() {
		let cap = 10;
		let dm = DataModel::with_capacity(TEST_NAMESPACE, cap);

		assert_eq!(dm.local_free_idx, Index::default());
		assert!(dm.data.capacity() >= cap);
		assert!(dm.pending.is_empty());
	}

	#[test]
	fn test_spawn() {
		let mut dm = DataModel::new(TEST_NAMESPACE);
		// Sanity checks
		assert_eq!(dm.local_free_idx, Index::default());
		assert!(dm.pending.is_empty());

		// Spawn the first entity
		let expected_state1 = bytes::Bytes::from_static(&[1]);
		let expected_index1 = Index::default();
		let entity1 = dm.spawn(expected_state1.clone());

		fn check_entity(
			dm: &DataModel,
			entity: EntityId,
			expected_state: &State,
			expected_idx: Index,
		) {
			// Check expected values of entity
			assert_eq!(entity.idx, expected_idx);
			assert_eq!(entity.namespace, TEST_NAMESPACE);
			// Check expected values of entity's state.
			assert_eq!(dm.get(entity), Ok(expected_state));
			// Check expected values of change detection
			assert_eq!(dm.pending.spawns[&entity], expected_state.clone());
			assert!(!dm.pending.despawns.contains_key(&entity));
			assert!(!dm.pending.mutations.contains_key(&entity));
		}

		check_entity(&dm, entity1, &expected_state1, expected_index1);
		// Check expected value of local_idx
		assert_eq!(dm.local_free_idx, entity1.idx.next());
		// Check expected values of change detection
		assert!(!dm.pending.is_empty());
		assert_eq!(dm.pending.spawns.len(), 1);
		assert_eq!(dm.pending.despawns.len(), 0);
		assert_eq!(dm.pending.mutations.len(), 0);

		// Spawn another entity
		let expected_state2 = bytes::Bytes::from_static(&[2]);
		let expected_index2 = Index::default().next();
		let entity2 = dm.spawn(expected_state2.clone());

		check_entity(&dm, entity1, &expected_state1, expected_index1);
		check_entity(&dm, entity2, &expected_state2, expected_index2);
		// Check expected value of local_idx
		assert_eq!(dm.local_free_idx, entity2.idx.next());
		// Check expected values of change detection
		assert!(!dm.pending.is_empty());
		assert_eq!(dm.pending.spawns.len(), 2);
		assert_eq!(dm.pending.despawns.len(), 0);
		assert_eq!(dm.pending.mutations.len(), 0);
	}

	#[test]
	fn test_flush_no_remote() {
		fn assert_dm_matches_local(dm: &DataModel, expected_local: &LocalChanges) {
			let pending = &dm.pending;
			// Sanity check, make sure that pending and data model are consistent.
			pending
				.spawns
				.iter()
				// Only check for existence on items that weren't despawned
				.filter(|(e, _bytes)| !pending.despawns.contains_key(e))
				.for_each(|(&entity, _bytes)| {
					assert!(dm.get(entity).is_ok(), "entity {entity:?}");
				});
			pending.despawns.iter().for_each(|(&entity, _bytes)| {
				assert_eq!(dm.get(entity), Err(EntityNotPresent), "entity {entity:?}");
			});
			pending.mutations.iter().for_each(|(&entity, _mutation)| {
				assert!(dm.get(entity).is_ok(), "entity {entity:?}")
			});

			// Actually compare pending to expected
			assert_eq!(pending.spawns, expected_local.spawns);
			assert_eq!(pending.despawns, expected_local.despawns);
			assert_eq!(pending.mutations.len(), expected_local.mutations.len());
			pending.mutations.iter().for_each(|(&entity, &mutation)| {
				let l_entry = &expected_local.mutations[&entity];
				assert_eq!(l_entry.0, mutation, "entity: {entity:?}");
				assert_eq!(l_entry.1, dm.get(entity).unwrap(), "entity: {entity:?}");
			});

			// compare data model states to expected
			let expected_states = {
				let mut states = expected_local.spawns.clone();
				states.extend(
					expected_local
						.mutations
						.iter()
						.map(|(&e, (_state_mutation, bytes))| (e, bytes.clone())),
				);
				states.retain(|e, _bytes| !expected_local.despawns.contains_key(e));
				states
			};
			assert_eq!(dm.data.len(), expected_states.len());
			for (e, entity_data) in dm.data.iter() {
				assert_eq!(entity_data.state, expected_states[e], "entity: {e:?}");
			}
		}

		let mut dm = DataModel::new(TEST_NAMESPACE);
		let mut expected_local = LocalChanges::default();

		fn check(dm: &DataModel, expected_local: &LocalChanges) {
			assert_dm_matches_local(dm, expected_local);
			let mut local_from_flush = LocalChanges::default();
			let remote = RemoteChanges::default();
			// Clone to avoid clearing pending local changes from datamodel needed in later steps.
			dm.clone().flush(&remote, &mut local_from_flush);
			assert_eq!(&local_from_flush, expected_local);
		}

		// Spawn e0
		let s0_a = State::from_static(b"s0_a");
		let e0 = dm.spawn(s0_a.clone());
		expected_local.spawns.insert(e0, s0_a.clone());
		check(&dm, &expected_local);

		// Spawn e1
		let s1_a = State::from_static(b"s1_a");
		let e1 = dm.spawn(s1_a.clone());
		assert_eq!(dm.get(e1).unwrap(), &s1_a);
		expected_local.spawns.insert(e1, s1_a);
		check(&dm, &expected_local);

		// Update e0 unreliably
		let s0_b = State::from_static(b"s0_b");
		dm.update(e0, s0_b.clone()).unwrap();
		expected_local
			.mutations
			.insert(e0, (StateMutation::Unreliable, s0_b));
		check(&dm, &expected_local);

		// Update e0 reliably
		let s0_c = State::from_static(b"s0_c");
		dm.update_reliable(e0, s0_c.clone()).unwrap();
		// Unreliable overwritten with Reliable, along with state.
		expected_local
			.mutations
			.insert(e0, (StateMutation::Reliable, s0_c));
		check(&dm, &expected_local);

		// Update e0 unreliably
		let s0_d = State::from_static(b"s0_d");
		dm.update(e0, s0_d.clone()).unwrap();
		// Reliable is not overwritten, but the state is.
		expected_local
			.mutations
			.insert(e0, (StateMutation::Reliable, s0_d.clone()));
		check(&dm, &expected_local);

		// Update e1 unreliably
		let s1_b = State::from_static(b"s1_b");
		dm.update(e1, s1_b.clone()).unwrap();
		expected_local
			.mutations
			.insert(e1, (StateMutation::Unreliable, s1_b));
		check(&dm, &expected_local);

		// Despawn e0
		dm.despawn(e0).unwrap();
		// The mutations should dissapear...
		let state_at_despawn = expected_local.mutations.remove(&e0).unwrap().1;
		// ...but the spawns should remain.
		assert_eq!(expected_local.spawns[&e0], s0_a);
		expected_local.despawns.insert(e0, state_at_despawn);
		check(&dm, &expected_local);
	}
}

#[cfg(test)]
mod test_index {
	use super::*;

	#[test]
	#[should_panic(expected = "ran out of available entities")]
	fn test_next_max_index_overflows() {
		Index(u32::MAX).next();
	}
}
