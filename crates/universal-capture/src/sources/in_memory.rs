use std::convert::Infallible;

use crate::InMemory;
use crate::Source;

impl Source<InMemory> for InMemory {
	type Error = Infallible;

	fn capture(&mut self, dest: &mut InMemory) -> Result<(), Self::Error> {
		std::mem::swap(dest, self);
		Ok(())
	}
}
