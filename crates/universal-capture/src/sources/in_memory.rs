use std::convert::Infallible;

use crate::Source;

impl<T> Source<Vec<T>> for Vec<T> {
	type Error = Infallible;

	fn capture(&mut self, dest: &mut Vec<T>) -> Result<(), Self::Error> {
		std::mem::swap(dest, self);
		Ok(())
	}
}
