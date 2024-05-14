//! An implementation of the [did:key] method.
//!
//! [did:key]: https://w3c-ccg.github.io/did-method-key/

use std::fmt::Display;

use crate::{
	uri::{DidMethod, DidUri},
	utf8bytes::Utf8Bytes,
};

/// An implementation of the `did:key` method. See the [module](self) docs for more
/// info.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DidKey {
	/// The string representation of the DID.
	s: Utf8Bytes,
	/// The substring for method-specific-id. This is a range index into `s`.
	method_specific_id: std::ops::RangeFrom<usize>,
}

impl DidKey {
	const PREFIX: &'static str = "did:key:";

	/// Gets the buffer representing the uri as a str.
	pub fn as_str(&self) -> &str {
		self.s.as_str()
	}

	/// Gets the buffer representing the uri as a byte slice.
	pub fn as_slice(&self) -> &[u8] {
		self.s.as_slice()
	}

	/// Gets the buffer representing the uri as a reference counted slice that
	/// is guaranteed to be utf8.
	pub fn as_utf8_bytes(&self) -> &Utf8Bytes {
		&self.s
	}
}

impl TryFrom<DidUri> for DidKey {
	type Error = FromUriError;

	fn try_from(value: DidUri) -> Result<Self, Self::Error> {
		let m = value.method();
		if m != DidMethod::Key {
			return Err(FromUriError::WrongMethod(m));
		}
		debug_assert_eq!(
			value.as_slice().len() - value.method_specific_id().as_slice().len(),
			Self::PREFIX.len(),
			"sanity check that prefix has expected length"
		);

		Ok(Self {
			s: value.as_utf8_bytes().clone(),
			method_specific_id: (Self::PREFIX.len()..),
		})
	}
}

#[derive(thiserror::Error, Debug)]
pub enum FromUriError {
	#[error("Expected \"key\" method but got {0:?}")]
	WrongMethod(DidMethod),
}

impl Display for DidKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_str().fmt(f)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	use eyre::WrapErr;
	use std::str::FromStr;

	fn common_examples() -> Vec<DidKey> {
		let p = DidKey::PREFIX;
		let make_example = |s| DidKey {
			s: format!("{p}{s}").into(),
			method_specific_id: (p.len()..),
		};
		["deadbeef123", "yeet"]
			.into_iter()
			.map(make_example)
			.collect()
	}

	#[test]
	fn test_try_from_uri() -> eyre::Result<()> {
		for example in common_examples() {
			let uri = DidUri::from_str(example.as_str())
				.wrap_err_with(|| format!("failed to parse DidUri from {example}"))?;
			let key_from_uri = DidKey::try_from(uri.clone())
				.wrap_err_with(|| format!("failed to parse DidKey from {uri}"))?;
			assert_eq!(uri.as_str(), key_from_uri.as_str());
		}
		Ok(())
	}
}
