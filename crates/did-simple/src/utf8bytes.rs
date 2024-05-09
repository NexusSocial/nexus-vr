use std::fmt::Display;

use bytes::Bytes;

/// Wrapper around [`Bytes`] which is guaranteed to be UTF-8.
/// Like `Bytes`, it is cheaply cloneable and facilitates zero-copy.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Utf8Bytes(Bytes);

impl Utf8Bytes {
	pub fn as_str(&self) -> &str {
		// TODO: Consider changing to unsafe later, because this check is entirely unecessary.
		std::str::from_utf8(self.0.as_ref()).expect("infallible")
	}

	pub fn as_slice(&self) -> &[u8] {
		self.0.as_ref()
	}

	pub fn bytes(&self) -> &Bytes {
		&self.0
	}

	pub fn into_inner(self) -> Bytes {
		self.0
	}

	/// Same as [`Bytes::split_off()`], but panics if slicing doesn't result in valid utf8.
	///
	/// Afterwards `self` contains elements `[0, at)`, and the returned `Utf8Bytes`
	/// contains elements `[at, len)`.
	pub fn split_off(&mut self, at: usize) -> Self {
		assert!(
			self.as_str().is_char_boundary(at),
			"slicing would have created invalid UTF-8!"
		);
		Self(self.0.split_off(at))
	}

	/// Same as [`Bytes::split_to()`], but panics if slicing doesn't result in valid utf8.
	///
	/// Afterwards `self` contains elements `[at, len)`, and the returned `Utf8Bytes`
	/// contains elements `[0, at)`.
	pub fn split_to(&mut self, at: usize) -> Self {
		assert!(
			self.as_str().is_char_boundary(at),
			"slicing would have created invalid UTF-8!"
		);
		Self(self.0.split_to(at))
	}
}

impl AsRef<[u8]> for Utf8Bytes {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

impl From<String> for Utf8Bytes {
	/// This is zero-copy, and skips UTF-8 checks.
	fn from(value: String) -> Self {
		Self(Bytes::from(value))
	}
}

impl From<&'static str> for Utf8Bytes {
	/// This is zero-copy, and skips UTF-8 checks.
	fn from(value: &'static str) -> Self {
		Self(Bytes::from_static(value.as_bytes()))
	}
}

impl TryFrom<Bytes> for Utf8Bytes {
	type Error = std::str::Utf8Error;

	/// This is zero-copy, and performs UTF-8 checks.
	fn try_from(value: Bytes) -> Result<Self, Self::Error> {
		let _s = std::str::from_utf8(value.as_ref())?;
		Ok(Self(value))
	}
}

impl Display for Utf8Bytes {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_str().fmt(f)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	const STRINGS: &[&str] = &["foobar", "", "\0", "Yellow ඞ Sus😂"];

	#[test]
	fn test_from_str() {
		for &s in STRINGS {
			let ub = Utf8Bytes::from(s);
			assert_eq!(s, ub.as_str());
			assert_eq!(s.as_bytes(), ub.0);
		}
	}

	#[test]
	fn test_from_string() {
		for &s in STRINGS {
			let s: String = s.to_owned();
			let ub = Utf8Bytes::from(s.clone());
			assert_eq!(s, ub.as_str());
			assert_eq!(s.as_bytes(), ub.0);
		}
	}

	#[test]
	fn test_from_bytes() {
		for &s in STRINGS {
			let b: Bytes = Bytes::from(s);
			let ub =
				Utf8Bytes::try_from(b.clone()).expect("failed conversion from bytes");
			assert_eq!(s, ub.as_str());
			assert_eq!(s.as_bytes(), ub.0);
			assert_eq!(ub.0, b);
		}
	}

	#[test]
	fn test_display() {
		for &s in STRINGS {
			let ub = Utf8Bytes::from(s);
			assert_eq!(s, format!("{ub}"))
		}
	}

	#[test]
	fn test_split_off() {
		for &s in STRINGS {
			let validity: Vec<bool> =
				(0..s.len()).map(|idx| s.is_char_boundary(idx)).collect();
			for pos in 0..s.len() {
				let mut original = Utf8Bytes::from(s);
				let result = std::panic::catch_unwind(move || {
					let ret = original.split_off(pos);
					(original, ret)
				});
				let is_valid = validity[pos];

				if let Ok((original, ret)) = result {
					assert_eq!(&s[0..pos], original.as_str());
					assert_eq!(&s[pos..], ret.as_str());
					assert!(
						is_valid,
						"split_off did not panic, so {pos} should be valid"
					);
				} else {
					assert!(
						!is_valid,
						"split_off panicked, so {pos} should not be valid"
					);
				}
			}
		}
	}

	#[test]
	fn test_split_to() {
		for &s in STRINGS {
			let validity: Vec<bool> =
				(0..s.len()).map(|idx| s.is_char_boundary(idx)).collect();
			for pos in 0..s.len() {
				let mut original = Utf8Bytes::from(s);
				let result = std::panic::catch_unwind(move || {
					let ret = original.split_to(pos);
					(original, ret)
				});
				let is_valid = validity[pos];

				if let Ok((original, ret)) = result {
					assert_eq!(&s[0..pos], ret.as_str());
					assert_eq!(&s[pos..], original.as_str());
					assert!(
						is_valid,
						"split_to did not panic, so {pos} should be valid"
					);
				} else {
					assert!(
						!is_valid,
						"split_to panicked, so {pos} should not be valid"
					);
				}
			}
		}
	}
}
