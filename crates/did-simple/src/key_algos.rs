use crate::varint::encode_varint;

/// A key algorithm.
pub trait KeyAlgo {
	fn pub_key_size(&self) -> usize;
	fn multicodec_value(&self) -> u16;
}

/// A key algorithm that is known statically, at compile time.
pub trait StaticKeyAlgo: KeyAlgo {
	const PUB_KEY_SIZE: usize;
	const MULTICODEC_VALUE: u16;
	const MULTICODEC_VALUE_ENCODED: &'static [u8] =
		encode_varint(Self::MULTICODEC_VALUE).as_slice();
}

impl<T: StaticKeyAlgo> KeyAlgo for T {
	fn pub_key_size(&self) -> usize {
		Self::PUB_KEY_SIZE
	}

	fn multicodec_value(&self) -> u16 {
		Self::MULTICODEC_VALUE
	}
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Ed25519;

impl StaticKeyAlgo for Ed25519 {
	const PUB_KEY_SIZE: usize = 32;
	const MULTICODEC_VALUE: u16 = 0xED;
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum DynKeyAlgo {
	Ed25519,
}

impl PartialEq<Ed25519> for DynKeyAlgo {
	fn eq(&self, _other: &Ed25519) -> bool {
		*self == DynKeyAlgo::Ed25519
	}
}

impl KeyAlgo for DynKeyAlgo {
	fn pub_key_size(&self) -> usize {
		match self {
			Self::Ed25519 => Ed25519::PUB_KEY_SIZE,
		}
	}

	fn multicodec_value(&self) -> u16 {
		match self {
			Self::Ed25519 => Ed25519::MULTICODEC_VALUE,
		}
	}
}
