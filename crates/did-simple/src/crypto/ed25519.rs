//! Implementation of the ed25519ph signing algorithm.

use curve25519_dalek::edwards::CompressedEdwardsY;

use super::Context;
use crate::key_algos::StaticSigningAlgo as _;

pub use ed25519_dalek::{ed25519::Signature, Digest, Sha512, SignatureError};

/// Re-exported for lower level control
pub use ed25519_dalek;

/// An ed25519 public key. Used for verifying messages.
///
/// We recommend deserializing bytes into this type using
/// [`Self::try_from_bytes()`]. Then you can either use this type, which has
/// simpler function signatures, or you can call [`Self::into_inner()`] and use
/// the lower level [`ed25519_dalek`] crate directly, which is slightly less
/// opinionated and has more customization of options made available.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct VerifyingKey(ed25519_dalek::VerifyingKey);

impl VerifyingKey {
	pub const LEN: usize = Self::key_len();

	/// Instantiates `PubKey` from some bytes. Performs all necessary validation
	/// that the key is valid and of sufficient strength.
	///
	/// Note that we will reject any keys that are too weak (aka low order).
	pub fn try_from_bytes(bytes: &[u8; Self::LEN]) -> Result<Self, TryFromBytesError> {
		let compressed_edwards = CompressedEdwardsY(bytes.to_owned());
		let Some(edwards) = compressed_edwards.decompress() else {
			return Err(TryFromBytesError::NotOnCurve);
		};
		let key = ed25519_dalek::VerifyingKey::from(edwards);
		if key.is_weak() {
			return Err(TryFromBytesError::WeakKey);
		}
		Ok(Self(key))
	}

	// TODO: Turn this into inline const when that feature stabilizes
	const fn key_len() -> usize {
		let len = crate::key_algos::Ed25519::VERIFYING_KEY_LEN;
		assert!(len == ed25519_dalek::PUBLIC_KEY_LENGTH);
		len
	}

	pub fn into_inner(self) -> ed25519_dalek::VerifyingKey {
		self.0
	}

	/// Verifies `message` using the ed25519ph algorithm.
	///
	/// # Example
	/// ```
	/// use did_simple::crypto::{Context, ed25519::{SigningKey, VerifyingKey}};
	///
	/// let signing_key = SigningKey::random();
	/// let verifying_key = signing_key.verifying_key();
	/// const CTX: Context = Context::from_bytes("MySuperCoolProtocol".as_bytes());
	///
	/// let msg = "everyone can read and verify this message";
	/// let sig = signing_key.sign(msg, CTX);
	///
	/// assert!(verifying_key.verify(msg, CTX, &sig).is_ok());
	/// ```
	pub fn verify(
		&self,
		message: impl AsRef<[u8]>,
		context: Context,
		signature: &Signature,
	) -> Result<(), SignatureError> {
		let digest = Sha512::new().chain_update(message);
		self.verify_digest(digest, context, signature)
	}

	/// Same as `verify`, but allows you to populate `message_digest` separately
	/// from signing.
	///
	/// This can be useful if for example, it is undesirable to buffer the message
	/// into a single slice, or the message is being streamed asynchronously.
	/// You can instead update the digest chunk by chunk, and pass the digest
	/// in after you are done reading all the data.
	///
	/// # Example
	///
	/// ```
	/// use did_simple::crypto::{Context, ed25519::{Sha512, Digest, SigningKey, VerifyingKey}};
	///
	/// let signing_key = SigningKey::random();
	/// let verifying_key = signing_key.verifying_key();
	/// const CTX: Context = Context::from_bytes("MySuperCoolProtocol".as_bytes());
	///
	/// let sig = signing_key.sign("this is my message", CTX);
	///
	/// let mut digest = Sha512::new();
	/// digest.update("this is ");
	/// digest.update("my message");
	/// assert!(verifying_key.verify_digest(digest, CTX, &sig).is_ok());
	pub fn verify_digest(
		&self,
		message_digest: Sha512,
		context: Context,
		signature: &Signature,
	) -> Result<(), SignatureError> {
		self.0
			.verify_prehashed_strict(message_digest, Some(context.0), signature)
	}
}

impl TryFrom<ed25519_dalek::VerifyingKey> for VerifyingKey {
	type Error = TryFromBytesError;

	fn try_from(value: ed25519_dalek::VerifyingKey) -> Result<Self, Self::Error> {
		Self::try_from_bytes(value.as_bytes())
	}
}

/// An ed25519 private key. Used for signing messages.
#[derive(Debug)]
pub struct SigningKey(ed25519_dalek::SigningKey);

impl SigningKey {
	pub const LEN: usize = Self::key_len();

	pub fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
		let signing = ed25519_dalek::SigningKey::from_bytes(bytes);
		let _pub = VerifyingKey::try_from(signing.verifying_key())
			.expect("this should never fail. if it does, please open an issue");
		Self(signing)
	}

	// TODO: Turn this into inline const when that feature stabilizes
	const fn key_len() -> usize {
		let len = crate::key_algos::Ed25519::SIGNING_KEY_LEN;
		assert!(len == ed25519_dalek::SECRET_KEY_LENGTH);
		len
	}

	pub fn into_inner(self) -> ed25519_dalek::SigningKey {
		self.0
	}

	/// Generates a new random [`SigningKey`].
	#[cfg(feature = "random")]
	pub fn random() -> Self {
		let mut csprng = rand_core::OsRng;
		Self(ed25519_dalek::SigningKey::generate(&mut csprng))
	}

	/// Generates a new random [`SigningKey`] from the given RNG.
	#[cfg(feature = "random")]
	pub fn random_from_rng<R: rand_core::CryptoRngCore + ?Sized>(rng: &mut R) -> Self {
		Self(ed25519_dalek::SigningKey::generate(rng))
	}

	/// Gets the public [`VerifyingKey`] that corresponds to this private [`SigningKey`].
	pub fn verifying_key(&self) -> VerifyingKey {
		VerifyingKey::try_from(self.0.verifying_key())
			.expect("this should never fail. if it does, please open an issue")
	}

	/// Signs `message` using the ed25519ph algorithm.
	///
	/// # Example
	///
	/// ```
	/// use did_simple::crypto::{Context, ed25519::{SigningKey, VerifyingKey}};
	///
	/// let signing_key = SigningKey::random();
	/// let verifying_key = signing_key.verifying_key();
	/// const CTX: Context = Context::from_bytes("MySuperCoolProtocol".as_bytes());
	///
	/// let msg = "everyone can read and verify this message";
	/// let sig = signing_key.sign(msg, CTX);
	///
	/// assert!(verifying_key.verify(msg, CTX, &sig).is_ok());
	/// ```
	pub fn sign(&self, message: impl AsRef<[u8]>, context: Context) -> Signature {
		let digest = Sha512::new().chain_update(message);
		self.sign_digest(digest, context)
	}

	/// Same as `sign`, but allows you to populate `message_digest` separately
	/// from signing.
	///
	/// This can be useful if for example, it is undesirable to buffer the message
	/// into a single slice, or the message is being streamed asynchronously.
	/// You can instead update the digest chunk by chunk, and pass the digest
	/// in after you are done reading all the data.
	///
	/// # Example
	///
	/// ```
	/// use did_simple::crypto::{Context, ed25519::{Sha512, Digest, SigningKey, VerifyingKey}};
	///
	/// let signing_key = SigningKey::random();
	/// let verifying_key = signing_key.verifying_key();
	/// const CTX: Context = Context::from_bytes("MySuperCoolProtocol".as_bytes());
	///
	/// let mut digest = Sha512::new();
	/// digest.update("this is ");
	/// digest.update("my message");
	/// let sig = signing_key.sign_digest(digest, CTX);
	///
	/// assert!(verifying_key.verify("this is my message", CTX, &sig).is_ok());
	pub fn sign_digest(&self, message_digest: Sha512, context: Context) -> Signature {
		self.0
			.sign_prehashed(message_digest, Some(context.0))
			.expect("this should never fail. if it does, please open an issue")
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

#[cfg(test)]
mod test {}
