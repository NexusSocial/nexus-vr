mod entity_map;

pub use self::entity_map::{Entity, StaleEntityIdErr};

use self::entity_map::EntityMap;

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

/// Elements of
#[derive(Debug, Clone, Default)]
pub struct DataModel {
	data: EntityMap<EntityData>,
	changes: EntityMap<Changed>,
}

impl DataModel {
	pub fn new() -> Self {
		Self {
			data: EntityMap::new(),
			// TODO: Create a secondary map. Generation doesn't need to be re-checked.
			changes: EntityMap::new(),
		}
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			data: EntityMap::with_capacity(cap),
			changes: EntityMap::with_capacity(cap),
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
	) -> Result<(), StaleEntityIdErr> {
		self.update_inner(entity, state, Changed::ReliableMutation)
	}

	pub fn update(
		&mut self,
		entity: Entity,
		state: State,
	) -> Result<(), StaleEntityIdErr> {
		self.update_inner(entity, state, Changed::UnreliableMutation)
	}

	fn update_inner(
		&mut self,
		entity: Entity,
		state: State,
		changed: Changed,
	) -> Result<(), StaleEntityIdErr> {
		if let Some(old) = self.data.get_mut(entity)? {
			old.state = state;
		} else {
			self.data
				.insert(
					entity,
					EntityData {
						state,
						..Default::default()
					},
				)
				.expect("already checked generation");
		}
		self.changes
			.insert(entity, changed)
			.expect("already checked generation");
		Ok(())
	}

	pub fn get(&self, entity: Entity) -> Result<Option<&State>, StaleEntityIdErr> {
		self.data
			.get(entity)
			.map(|data| data.map(|data| &data.state))
	}

	pub fn remove(&mut self, entity: Entity) -> Result<(), StaleEntityIdErr> {
		self.data.remove(entity)?;
		self.changes
			.insert(entity, Changed::Deleted)
			.expect("already checked generation");
		Ok(())
	}

	/// Returns the priorities as `(send, recv)`.
	pub fn priority(
		&mut self,
		entity: Entity,
	) -> Result<(Priority, Priority), EntityAccessErr> {
		self.data
			.get_mut(entity)?
			.ok_or(EntityAccessErr::NotPresent)
			.map(|data| (data.send_prio, data.recv_prio))
	}

	/// Returns mutable refs to the priorities as `(send, recv)`.
	pub fn priority_mut(
		&mut self,
		entity: Entity,
	) -> Result<(&mut Priority, &mut Priority), EntityAccessErr> {
		self.data
			.get_mut(entity)?
			.ok_or(EntityAccessErr::NotPresent)
			.map(|data| (&mut data.send_prio, &mut data.recv_prio))
	}
}

#[derive(thiserror::Error, Debug)]
pub enum EntityAccessErr {
	#[error(transparent)]
	Stale(#[from] StaleEntityIdErr),
	#[error("entity not present")]
	NotPresent,
}
