#[macro_export]
macro_rules! unwrap_or_continue {
	($e:expr) => {
		if let Some(e) = $e {
			e
		} else {
			continue;
		}
	};
}
pub use unwrap_or_continue;
