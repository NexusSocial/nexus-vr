use std::str::FromStr;

use bytes::Bytes;

use crate::utf8bytes::Utf8Bytes;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum DidMethod {
	Key,
	Web,
}

impl FromStr for DidMethod {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"key" => Self::Key,
			"web" => Self::Web,
			"" => return Err(ParseError::MissingMethod),
			_ => return Err(ParseError::UnknownMethod),
		})
	}
}

/// Helper type to access data in the method-specific-id of a [`DidUri`].
pub struct MethodSpecificId<'a>(&'a DidUri);

impl MethodSpecificId<'_> {
	pub fn as_str(&self) -> &str {
		&(self.0.as_str()[self.0.method_specific_id.clone()])
	}

	pub fn as_slice(&self) -> &[u8] {
		&(self.0.s.as_slice()[self.0.method_specific_id.clone()])
	}

	pub fn utf8_bytes(&self) -> Utf8Bytes {
		self.0.s.clone().split_off(self.0.method_specific_id.start)
	}
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DidUri {
	method: DidMethod,
	/// The string representation of the DID.
	s: Utf8Bytes,
	/// The substring for method-specific-id. This is a range index into `s`.
	method_specific_id: std::ops::RangeFrom<usize>,
}

impl DidUri {
	/// Gets the buffer representing the uri as a str.
	pub fn as_str(&self) -> &str {
		self.s.as_str()
	}

	/// Gets the buffer representing the uri as a byte slice.
	pub fn as_slice(&self) -> &[u8] {
		self.s.as_slice()
	}

	/// Gets the buffer representing the uri as a byte slice that is guaranteed to be utf8.
	pub fn utf8_bytes(&self) -> &Utf8Bytes {
		&self.s
	}

	/// Gets the buffer representing the uri as bytes.
	pub fn bytes(&self) -> &Bytes {
		self.s.bytes()
	}

	/// The method of the did.
	pub fn method(&self) -> DidMethod {
		self.method
	}

	/// Method-specific identity info.
	pub fn method_specific_id(&self) -> MethodSpecificId {
		MethodSpecificId(self)
	}

	pub fn into_inner(self) -> Utf8Bytes {
		self.s
	}
}

impl FromStr for DidUri {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (method, remaining) = s
			.strip_prefix("did:")
			.ok_or(ParseError::InvalidScheme)?
			.split_once(':')
			.ok_or(ParseError::MissingMethod)?;
		let method = DidMethod::from_str(method)?;
		let start_idx = s.len() - remaining.len();

		Ok(DidUri {
			method,
			s: Utf8Bytes::from(s.to_owned()),
			method_specific_id: (start_idx..),
		})
	}
}

impl TryFrom<String> for DidUri {
	type Error = ParseError;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		let (method, remaining) = s
			.strip_prefix("did:")
			.ok_or(ParseError::InvalidScheme)?
			.split_once(':')
			.ok_or(ParseError::MissingMethod)?;
		let method = DidMethod::from_str(method)?;
		let start_idx = s.len() - remaining.len();

		Ok(DidUri {
			method,
			s: Utf8Bytes::from(s),
			method_specific_id: (start_idx..),
		})
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("expected the did: scheme")]
	InvalidScheme,
	#[error("expected did:method, but method was not present")]
	MissingMethod,
	#[error("encountered unknown did:method")]
	UnknownMethod,
}

#[cfg(test)]
mod test {
	use super::*;
	use eyre::{Result, WrapErr};

	#[test]
	fn test_parse() -> Result<()> {
		let test_cases = [DidUri {
			method: DidMethod::Key,
			s: String::from("did:key:123456").into(),
			method_specific_id: (8..),
		}];
		for expected in test_cases {
			let s = expected.s.as_str().to_owned();
			let from_str = DidUri::from_str(&s).wrap_err("failed to from_str")?;
			let try_from = DidUri::try_from(s).wrap_err("failed to try_from")?;
			assert_eq!(from_str, try_from);
			assert_eq!(from_str, expected);
		}
		Ok(())
	}
}
