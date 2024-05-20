use crate::varint::encode_varint;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum KeyAlgo {
	Ed25519,
}

impl KeyAlgo {
	pub fn pub_key_len(&self) -> usize {
		match self {
			Self::Ed25519 => Ed25519::PUB_KEY_LEN,
		}
	}
}

// ---- internal code ----

/// A key algorithm that is known statically, at compile time.
pub(crate) trait StaticKeyAlgo {
	const PUB_KEY_LEN: usize;
	const MULTICODEC_VALUE: u16;
	const MULTICODEC_VALUE_ENCODED: &'static [u8] =
		encode_varint(Self::MULTICODEC_VALUE).as_slice();
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub(crate) struct Ed25519;

impl StaticKeyAlgo for Ed25519 {
	const PUB_KEY_LEN: usize = 32;
	const MULTICODEC_VALUE: u16 = 0xED;
}

impl PartialEq<Ed25519> for KeyAlgo {
	fn eq(&self, _other: &Ed25519) -> bool {
		*self == KeyAlgo::Ed25519
	}
}
