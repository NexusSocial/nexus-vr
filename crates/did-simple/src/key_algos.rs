use ref_cast::RefCast;

use crate::varint::encode_varint;

/// A key algorithm.
pub trait KeyAlgo {
	type PubKey: AsRef<[u8]>;
	fn pub_key_size(&self) -> usize;
	fn multicodec_value(&self) -> u16;
}

/// A key algorithm that is known statically, at compile time.
pub trait StaticKeyAlgo: KeyAlgo {
	const PUB_KEY_SIZE: usize;
	const MULTICODEC_VALUE: u16;
	const MULTICODEC_VALUE_ENCODED: &'static [u8] =
		encode_varint(Self::MULTICODEC_VALUE).as_slice();
	type PubKeyArray: AsRef<[u8]>;
}

impl<T: StaticKeyAlgo> KeyAlgo for T {
	type PubKey = T::PubKeyArray;

	fn pub_key_size(&self) -> usize {
		Self::PUB_KEY_SIZE
	}

	fn multicodec_value(&self) -> u16 {
		Self::MULTICODEC_VALUE
	}
}

#[derive(RefCast)]
#[repr(transparent)]
pub struct PubKey<T: StaticKeyAlgo>(pub T::PubKeyArray);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Ed25519;

impl StaticKeyAlgo for Ed25519 {
	const PUB_KEY_SIZE: usize = 32;
	const MULTICODEC_VALUE: u16 = 0xED;
	type PubKeyArray = [u8; Self::PUB_KEY_SIZE];
}

impl PartialEq<Ed25519> for DynKeyAlgo {
	fn eq(&self, _other: &Ed25519) -> bool {
		*self == DynKeyAlgo::Ed25519
	}
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum DynKeyAlgo {
	Ed25519,
}

impl KeyAlgo for DynKeyAlgo {
	type PubKey = DynPubKey;

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

#[non_exhaustive]
pub enum DynPubKey {
	Ed25519(PubKey<Ed25519>),
}

impl From<PubKey<Ed25519>> for DynPubKey {
	fn from(value: PubKey<Ed25519>) -> Self {
		Self::Ed25519(value)
	}
}

impl AsRef<[u8]> for DynPubKey {
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::Ed25519(k) => k.0.as_ref(),
		}
	}
}

#[non_exhaustive]
pub enum DynPubKeyRef<'a> {
	Ed25519(&'a PubKey<Ed25519>),
}

impl<'a> From<&'a PubKey<Ed25519>> for DynPubKeyRef<'a> {
	fn from(value: &'a PubKey<Ed25519>) -> Self {
		Self::Ed25519(value)
	}
}

impl AsRef<[u8]> for DynPubKeyRef<'_> {
	fn as_ref(&self) -> &[u8] {
		match *self {
			Self::Ed25519(k) => k.0.as_ref(),
		}
	}
}
