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
	pub trait Sealed {}

	impl<T> Sealed for Vec<T> {}
}
