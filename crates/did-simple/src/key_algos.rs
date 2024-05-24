use crate::varint::encode_varint;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum KeyAlgo {
	Ed25519,
}

impl KeyAlgo {
	pub fn verifying_key_len(&self) -> usize {
		match self {
			Self::Ed25519 => Ed25519::VERIFYING_KEY_LEN,
		}
	}

	pub fn signing_key_len(&self) -> usize {
		match self {
			Self::Ed25519 => Ed25519::SIGNING_KEY_LEN,
		}
	}
}

// ---- internal code ----

/// A signing algorithm that is known statically, at compile time.
pub(crate) trait StaticSigningAlgo {
	/// The length of the public verifying key.
	const VERIFYING_KEY_LEN: usize;
	/// The length of the private signing key.
	const SIGNING_KEY_LEN: usize;
	const MULTICODEC_VALUE: u16;
	const MULTICODEC_VALUE_ENCODED: &'static [u8] =
		encode_varint(Self::MULTICODEC_VALUE).as_slice();
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub(crate) struct Ed25519;

impl StaticSigningAlgo for Ed25519 {
	const VERIFYING_KEY_LEN: usize = 32;
	const SIGNING_KEY_LEN: usize = 32;
	const MULTICODEC_VALUE: u16 = 0xED;
}

impl PartialEq<Ed25519> for KeyAlgo {
	fn eq(&self, _other: &Ed25519) -> bool {
		*self == KeyAlgo::Ed25519
	}
}
