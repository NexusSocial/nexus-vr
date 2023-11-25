use crate::InMemory;
use crate::Source;
use eyre::{Report, WrapErr};

use dxcapture::Capture;

impl Source<InMemory> for Capture {
	type Error = Report;

	fn capture(&mut self, dest: &mut InMemory) -> Result<(), Self::Error> {
		let mut raw = self.get_raw_frame().wrap_err("Failed to capture frame")?;
		std::mem::swap(&mut dest.data, &mut raw.data);
		dest.dims.width = raw.width as usize;
		dest.dims.height = raw.height as usize;
		Ok(())
	}
}

impl crate::private::Sealed for Capture {}
