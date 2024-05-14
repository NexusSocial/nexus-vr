//! An implementation of the [did:key] method.
//!
//! [did:key]: https://w3c-ccg.github.io/did-method-key/

use std::fmt::Display;

use crate::{
	uri::{DidMethod, DidUri},
	utf8bytes::Utf8Bytes,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum PubKeyKind {}

/// An implementation of the `did:key` method. See the [module](self) docs for more
/// info.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DidKey {
	/// The string representation of the DID.
	s: Utf8Bytes,
	/// The decoded multibase portion of the DID.
	decoded: Vec<u8>,
}

impl DidKey {
	pub const PREFIX: &'static str = "did:key:";

	/// Gets the buffer representing the did:key uri as a str.
	pub fn as_str(&self) -> &str {
		self.s.as_str()
	}

	/// Gets the buffer representing the did:key uri as a byte slice.
	pub fn as_slice(&self) -> &[u8] {
		self.s.as_slice()
	}

	/// Gets the buffer representing the did:key uri as a reference counted slice
	/// that is guaranteed to be utf8.
	pub fn as_utf8_bytes(&self) -> &Utf8Bytes {
		&self.s
	}
}

fn decode_multibase(
	s: &Utf8Bytes,
	out_buf: &mut Vec<u8>,
) -> Result<(), MultibaseDecodeError> {
	out_buf.clear();
	// did:key only uses base58-btc, so its not actually any arbitrary multibase.
	let multibase_part = &s.as_slice()[DidKey::PREFIX.len()..];
	// the first character should always be 'z'
	let base = multibase_part[0];
	if base != b'z' {
		return Err(MultibaseDecodeError::WrongBase(base));
	}
	bs58::decode(&multibase_part[1..])
		.with_alphabet(bs58::Alphabet::BITCOIN)
		.onto(out_buf)?;
	Ok(())
}
#[derive(thiserror::Error, Debug)]
pub enum MultibaseDecodeError {
	#[error(
		"Expected \"base58-btc\" encoding which should be identified in multibase as ascii 'z' (0x7a) but got {0:x}"
	)]
	WrongBase(u8),
	#[error(transparent)]
	Bs58(#[from] bs58::decode::Error),
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

		let s = value.as_utf8_bytes().clone();
		let mut decoded = Vec::new();
		decode_multibase(&s, &mut decoded)?;

		Ok(Self { s, decoded })
	}
}

#[derive(thiserror::Error, Debug)]
pub enum FromUriError {
	#[error("Expected \"key\" method but got {0:?}")]
	WrongMethod(DidMethod),
	#[error(transparent)]
	MultibaseDecode(#[from] MultibaseDecodeError),
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
	use hex_literal::hex;
	use std::str::FromStr;

	// From: https://w3c-ccg.github.io/did-method-key/#example-5
	fn ed25519_examples() -> &'static [&'static str] {
		&[
			"did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
			"did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
			"did:key:z6MknGc3ocHs3zdPiJbnaaqDi58NGb4pk1Sp9WxWufuXSdxf",
		]
	}

	#[test]
	fn test_try_from_uri() -> eyre::Result<()> {
		for &example in ed25519_examples() {
			let uri = DidUri::from_str(example)
				.wrap_err_with(|| format!("failed to parse DidUri from {example}"))?;
			assert_eq!(example, uri.as_str());
			let key_from_uri = DidKey::try_from(uri.clone())
				.wrap_err_with(|| format!("failed to parse DidKey from {uri}"))?;
			assert_eq!(example, key_from_uri.as_str());
		}
		Ok(())
	}

	#[test]
	fn test_decode_multibase() -> eyre::Result<()> {
		#[derive(Debug)]
		struct Example {
			decoded: &'static [u8],
			encoded: &'static str,
		}
		// from: https://datatracker.ietf.org/doc/html/draft-msporny-base58-03#section-5
		let examples = [
			Example {
				decoded: b"Hello World!",
				encoded: "2NEpo7TZRRrLZSi2U",
			},
			Example {
				decoded: b"The quick brown fox jumps over the lazy dog.",
				encoded: "USm3fpXnKG5EUBx2ndxBDMPVciP5hGey2Jh4NDv6gmeo1LkMeiKrLJUUBk6Z",
			},
			Example {
				decoded: &hex!("0000287fb4cd"),
				encoded: "11233QC4",
			},
		];

		let mut buf = Vec::new();
		for e @ Example { decoded, encoded } in examples {
			let s = format!("{}z{encoded}", DidKey::PREFIX).into();
			decode_multibase(&s, &mut buf)
				.wrap_err_with(|| format!("Failed to decode example {e:?}"))?;
			assert_eq!(buf, decoded, "failed comparison in example {e:?}");
		}

		Ok(())
	}
}
