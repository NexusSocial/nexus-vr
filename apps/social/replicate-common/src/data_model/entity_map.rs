use super::{Entity, State};

/// More efficient version of `HashMap<Entity, State>`
pub struct EntityMap<T> {
	v: Vec<T>,
}

impl<T> EntityMap<T> {
	pub fn new() -> Self {
		Self { v: Vec::new() }
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			v: Vec::with_capacity(cap),
		}
	}

	pub fn get(&self, e: Entity) -> Option<&State> {
		todo!()
	}

	pub fn insert(&mut self, e: Entity, s: State) -> Option<State> {
		todo!()
	}

	pub fn remove(&mut self, e: Entity) -> Option<State> {
		todo!()
	}
}
