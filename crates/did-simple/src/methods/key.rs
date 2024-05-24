//! An implementation of the [did:key] method.
//!
//! [did:key]: https://w3c-ccg.github.io/did-method-key/

use std::fmt::Display;

use crate::{
	key_algos::{Ed25519, KeyAlgo, StaticSigningAlgo},
	url::{DidMethod, DidUrl},
	utf8bytes::Utf8Bytes,
	varint::decode_varint,
};

/// An implementation of the `did:key` method. See the [module](self) docs for more
/// info.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DidKey {
	/// The string representation of the DID.
	s: Utf8Bytes,
	/// The decoded multibase portion of the DID.
	mb_value: Vec<u8>,
	key_algo: KeyAlgo,
	/// The index into [`Self::mb_value`] that is the public key.
	pubkey_bytes: std::ops::RangeFrom<usize>,
}

pub const PREFIX: &str = "did:key:";

impl DidKey {
	pub const PREFIX: &'static str = PREFIX;

	/// Gets the buffer representing the did:key url as a str.
	pub fn as_str(&self) -> &str {
		self.s.as_str()
	}

	/// Gets the buffer representing the did:key url as a byte slice.
	pub fn as_slice(&self) -> &[u8] {
		self.s.as_slice()
	}

	/// Gets the buffer representing the did:key url as a reference counted slice
	/// that is guaranteed to be utf8.
	pub fn as_utf8_bytes(&self) -> &Utf8Bytes {
		&self.s
	}

	pub fn key_algo(&self) -> KeyAlgo {
		self.key_algo
	}

	/// Gets the decoded bytes of the public key.
	pub fn pub_key(&self) -> &[u8] {
		let result = match self.key_algo {
			KeyAlgo::Ed25519 => &self.mb_value[self.pubkey_bytes.clone()],
		};
		debug_assert_eq!(result.len(), self.key_algo.verifying_key_len());
		result
	}
}

fn decode_multibase(
	s: &Utf8Bytes,
	out_buf: &mut Vec<u8>,
) -> Result<(), MultibaseDecodeError> {
	out_buf.clear();
	// did:key only uses base58-btc, so its not actually any arbitrary multibase.
	let multibase_part = &s.as_slice()[PREFIX.len()..];
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

impl TryFrom<DidUrl> for DidKey {
	type Error = FromUrlError;

	fn try_from(value: DidUrl) -> Result<Self, Self::Error> {
		let m = value.method();
		if m != DidMethod::Key {
			return Err(FromUrlError::WrongMethod(m));
		}
		debug_assert_eq!(
			value.as_slice().len() - value.method_specific_id().as_slice().len(),
			PREFIX.len(),
			"sanity check that prefix has expected length"
		);

		let s = value.as_utf8_bytes().clone();
		let mut decoded_multibase = Vec::new();
		decode_multibase(&s, &mut decoded_multibase)?;

		// tail bytes will end up being the pubkey bytes if everything passes validation
		let (multicodec_key_algo, tail_bytes) = decode_varint(&decoded_multibase)?;
		let (key_algo, pub_key_len) = match multicodec_key_algo {
			Ed25519::MULTICODEC_VALUE => (KeyAlgo::Ed25519, Ed25519::SIGNING_KEY_LEN),
			_ => return Err(FromUrlError::UnknownKeyAlgo(multicodec_key_algo)),
		};

		if tail_bytes.len() != pub_key_len {
			return Err(FromUrlError::MismatchedPubkeyLen(key_algo, pub_key_len));
		}

		let pubkey_bytes = (decoded_multibase.len() - pub_key_len)..;

		Ok(Self {
			s,
			mb_value: decoded_multibase,
			key_algo,
			pubkey_bytes,
		})
	}
}

#[derive(thiserror::Error, Debug)]
pub enum FromUrlError {
	#[error("Expected \"key\" method but got {0:?}")]
	WrongMethod(DidMethod),
	#[error(transparent)]
	MultibaseDecode(#[from] MultibaseDecodeError),
	#[error("unknown multicodec value for key algorithm: decoded varint as {0}")]
	UnknownKeyAlgo(u16),
	#[error(transparent)]
	Varint(#[from] crate::varint::DecodeError),
	#[error("{0:?} requires pubkeys of length {} but got {1} bytes", .0.verifying_key_len())]
	MismatchedPubkeyLen(KeyAlgo, usize),
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
	//0xed
	// From: https://w3c-ccg.github.io/did-method-key/#example-5
	fn ed25519_examples() -> &'static [&'static str] {
		&[
			"did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
			"did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
			"did:key:z6MknGc3ocHs3zdPiJbnaaqDi58NGb4pk1Sp9WxWufuXSdxf",
		]
	}

	#[test]
	fn test_try_from_url() -> eyre::Result<()> {
		for &example in ed25519_examples() {
			let url = DidUrl::from_str(example)
				.wrap_err_with(|| format!("failed to parse DidUrl from {example}"))?;
			assert_eq!(example, url.as_str());
			let key_from_url = DidKey::try_from(url.clone())
				.wrap_err_with(|| format!("failed to parse DidKey from {url}"))?;
			assert_eq!(example, key_from_url.as_str());
			assert_eq!(key_from_url.key_algo(), Ed25519);
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
			let s = format!("{}z{encoded}", PREFIX).into();
			decode_multibase(&s, &mut buf)
				.wrap_err_with(|| format!("Failed to decode example {e:?}"))?;
			assert_eq!(buf, decoded, "failed comparison in example {e:?}");
		}

		Ok(())
	}
}
