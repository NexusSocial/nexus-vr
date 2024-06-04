//! Provides the [`seed_rng`] macro.

use std::{hash::Hasher, marker::PhantomData};

/// Used instead of `rand`'s `SmallRng` because rand says that it
/// is not portable across platforms.
///
/// DO NOT USE FOR CRYPTO OMEGALUL
type InsecureRng = rand_xoshiro::Xoshiro128PlusPlus;

/// Uses the type name of `T` as the seed. Implements `default()` for use
/// in local structs.
pub(crate) struct SeededRng<T> {
	rng: InsecureRng,
	_phantom: PhantomData<T>,
}

impl<T: std::any::Any> Default for SeededRng<T> {
	fn default() -> Self {
		let seed_str = std::any::type_name::<T>();
		let mut hasher = bevy::utils::AHasher::default();
		hasher.write(seed_str.as_bytes());
		let rng = <InsecureRng as rand::SeedableRng>::seed_from_u64(hasher.finish());
		Self {
			rng,
			_phantom: Default::default(),
		}
	}
}

impl<T> rand::RngCore for SeededRng<T> {
	fn next_u32(&mut self) -> u32 {
		self.rng.next_u32()
	}

	fn next_u64(&mut self) -> u64 {
		self.rng.next_u64()
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		self.rng.fill_bytes(dest)
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
		self.rng.try_fill_bytes(dest)
	}
}

macro_rules! seed_rng {
	($ident:ident, $seed:ident) => {
		struct $seed;
		type $ident = $crate::rng::SeededRng<$seed>;
	};
}
pub(crate) use seed_rng;
