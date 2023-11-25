pub mod dests;
pub mod sources;

pub trait Dest: private::Sealed {}

pub trait Source<Dest>: private::Sealed {
	/// Error returned by [`Self::capture`]
	type Error;

	/// Captures a frame from the `Source` and puts it in `Dest`.
	fn capture(&mut self, dest: &mut Dest) -> Result<(), Self::Error>;
}

mod private {
	use super::InMemory;

	pub trait Sealed {}

	impl Sealed for InMemory {}
}

#[derive(Debug, Clone)]
pub struct InMemory {
	pub data: Vec<u8>,
	pub dims: Dims,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Dims {
	pub width: usize,
	pub height: usize,
}

impl Dims {
	pub const fn size(&self) -> usize {
		self.width * self.height
	}
}
