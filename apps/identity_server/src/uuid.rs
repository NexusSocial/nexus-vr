//! Mockable UUID generation.

use ::uuid::Uuid;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Handles generation of UUIDs. This is used instead of the uuid crate directly,
/// to better support deterministic UUID creation in tests.
#[derive(Debug)]
pub struct UuidProvider {
	#[cfg(not(test))]
	provider: ThreadLocalRng,
	#[cfg(test)]
	provider: Box<dyn UuidProviderT>,
}

impl UuidProvider {
	#[allow(dead_code)]
	pub fn new_thread_local() -> Self {
		Self {
			#[cfg(test)]
			provider: Box::new(ThreadLocalRng),
			#[cfg(not(test))]
			provider: ThreadLocalRng,
		}
	}

	/// Allows controlling the sequence of generated UUIDs. Only available in
	/// `cfg(test)`.
	#[cfg(test)]
	pub fn new_from_sequence(uuids: Vec<Uuid>) -> Self {
		Self {
			provider: Box::new(TestSequence::new(uuids)),
		}
	}

	#[inline]
	pub fn next_v4(&self) -> Uuid {
		self.provider.next_v4()
	}
}

impl Default for UuidProvider {
	fn default() -> Self {
		Self::new_thread_local()
	}
}

trait UuidProviderT: std::fmt::Debug + Send + Sync + 'static {
	fn next_v4(&self) -> Uuid;
}

#[derive(Debug)]
struct ThreadLocalRng;
impl UuidProviderT for ThreadLocalRng {
	fn next_v4(&self) -> Uuid {
		Uuid::new_v4()
	}
}

/// Provides UUIDs from a known sequence. Useful for tests.
#[derive(Debug)]
struct TestSequence {
	uuids: Vec<Uuid>,
	pos: AtomicUsize,
}
impl TestSequence {
	/// # Panics
	/// Panics if len of vec is 0
	#[allow(dead_code)]
	fn new(uuids: Vec<Uuid>) -> Self {
		assert!(!uuids.is_empty());
		Self {
			uuids,
			pos: AtomicUsize::new(0),
		}
	}
}

impl UuidProviderT for TestSequence {
	fn next_v4(&self) -> Uuid {
		let curr_pos = self.pos.fetch_add(1, Ordering::SeqCst) % self.uuids.len();
		self.uuids[curr_pos]
	}
}

fn _assert_bounds(p: UuidProvider) {
	fn helper(_p: impl std::fmt::Debug + Send + Sync + 'static) {}
	helper(p)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_sequence_order() {
		let uuids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
		let sequence = TestSequence::new(uuids.clone());
		for uuid in uuids {
			assert_eq!(uuid, sequence.next_v4());
		}
	}
}
