use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::VerifyingKey;

use crate::key_algos::StaticKeyAlgo as _;

/// An ed25519 public key.
#[allow(dead_code)]
pub struct PubKey(VerifyingKey);

impl PubKey {
	pub const LEN: usize = Self::key_len();

	/// Instantiates `PubKey` from some bytes. Performs all necessary validation
	/// that the key is valid and of sufficient strength.
	///
	/// Note that we will reject any keys that are too weak (aka low order).
	pub fn try_from(bytes: &[u8; Self::LEN]) -> Result<Self, TryFromBytesError> {
		let compressed_edwards = CompressedEdwardsY(bytes.to_owned());
		let Some(edwards) = compressed_edwards.decompress() else {
			return Err(TryFromBytesError::NotOnCurve);
		};
		let key = VerifyingKey::from(edwards);
		if key.is_weak() {
			return Err(TryFromBytesError::WeakKey);
		}
		Ok(Self(key))
	}

	// TODO: Turn this into inline const when that feature stabilizes
	const fn key_len() -> usize {
		let len = crate::key_algos::Ed25519::PUB_KEY_LEN;
		assert!(len == ed25519_dalek::PUBLIC_KEY_LENGTH);
		len
	}
}

#[derive(thiserror::Error, Debug)]
pub enum TryFromBytesError {
	#[error(
		"the provided bytes was not the y coordinate of a valid point on the curve"
	)]
	NotOnCurve,
	#[error("public key has a low order and is too weak, which would allow the key to generate signatures that work for almost any message. To prevent this, we reject weak keys.")]
	WeakKey,
}

/// Errors which may occur while processing signatures and keypairs.
#[derive(thiserror::Error, Debug)]
#[error("invalid signature")]
pub struct SignatureError(#[from] ed25519_dalek::SignatureError);
