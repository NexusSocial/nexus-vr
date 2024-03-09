use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
#[error("entity was previously deleted")]
pub struct StaleEntityIdErr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Index(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Generation(u32);

/// An identifier for an entity in the network datamodel. NOTE: This is not the
/// same as an ECS entity. This crate is completely independent of bevy.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity {
	idx: Index,
	gen: Generation,
}

#[derive(Debug, Clone)]
pub struct EntityMap<T>(HashMap<Index, (Generation, T)>);

impl<T> EntityMap<T> {
	pub fn new() -> Self {
		Self(HashMap::new())
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(HashMap::with_capacity(cap))
	}

	pub fn get(&self, entity: Entity) -> Result<Option<&T>, StaleEntityIdErr> {
		let Some((inner_gen, inner_val)) = self.0.get(&entity.idx) else {
			return Ok(None);
		};
		if *inner_gen < entity.gen {
			return Ok(None);
		}
		if *inner_gen == entity.gen {
			return Ok(Some(inner_val));
		}
		Err(StaleEntityIdErr)
	}

	pub fn get_mut(
		&mut self,
		entity: Entity,
	) -> Result<Option<&mut T>, StaleEntityIdErr> {
		let Some((inner_gen, inner_val)) = self.0.get_mut(&entity.idx) else {
			return Ok(None);
		};
		if *inner_gen < entity.gen {
			return Ok(None);
		}
		if *inner_gen == entity.gen {
			return Ok(Some(inner_val));
		}
		Err(StaleEntityIdErr)
	}

	pub fn insert(
		&mut self,
		entity: Entity,
		mut value: T,
	) -> Result<Option<T>, StaleEntityIdErr> {
		let Some((inner_gen, inner_val)) = self.0.get_mut(&entity.idx) else {
			self.0.insert(entity.idx, (entity.gen, value));
			return Ok(None);
		};
		if *inner_gen > entity.gen {
			return Err(StaleEntityIdErr);
		}
		std::mem::swap(inner_val, &mut value);
		*inner_gen = entity.gen;
		Ok(Some(value))
	}

	pub fn remove(&mut self, entity: Entity) -> Result<Option<T>, StaleEntityIdErr> {
		let Some((inner_gen, _inner_val)) = self.0.get(&entity.idx) else {
			return Ok(None);
		};
		if *inner_gen > entity.gen {
			return Err(StaleEntityIdErr);
		}
		Ok(Some(
			self.0
				.remove(&entity.idx)
				.expect("should never fail because we already checked for presence")
				.1,
		))
	}
}

impl<T> Default for EntityMap<T> {
	fn default() -> Self {
		EntityMap::new()
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn test_entity_map() {
		// TODO
	}
}
