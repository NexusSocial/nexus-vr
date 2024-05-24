//! Implementations of cryptographic operations

// Re-exports
#[cfg(feature = "random")]
pub use rand_core;

#[cfg(feature = "ed25519")]
pub mod ed25519;

/// The "context" for signing and verifying messages, which is used for domain
/// separation of message signatures. The context can be of length
/// `[Context::MIN_LEN..Context::MAX_LEN]`.
///
/// # Example
///
/// ```
/// use did_simple::crypto::Context;
/// const CTX: Context = Context::from_bytes("MySuperCoolProtocol".as_bytes());
/// ```
///
/// # What is the purpose of this?
///
/// Messages signed using one context cannot be verified under a different
/// context. This is important, because it prevents tricking someone into
/// signing a message for one use case, and it getting reused for another use
/// case.
///
/// # Can you give me an example of how not using a context can be bad?
///
/// Suppose that a scammer sends you a file and asks you to send it back to them
/// signed, to prove to them that you got the message. You naively comply, only
/// later to realize that the file you signed actually is a json message that
/// authorizes your bank to send funds to the scammer. If you reused the same
/// public key for sending messages as you do for authorizing bank transactions,
/// you just got robbed.
///
/// If instead the application you were using signed that message with a
/// "MySecureProtocolSendMessage" context, and your bank used "MySuperSafeBank",
/// your bank would have rejected the message signature when the scammer tried
/// to use it to authorize a funds transfer because the two contexts don't
/// match.
///
/// # But I *really* need to not use a context for this specific case 🥺
///
/// Most of the signing algorithms' `VerifyingKey`s expose an `into_inner`
/// method and reexport the cryptography crate they use. So you can just call
/// the relevant signing functions yourself with the lower level crate.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Context<'a>(&'a [u8]);

/// Helper just to allow same value in macro contexts as const contexts
macro_rules! ctx_len {
	("max") => {
		255
	};
	("min") => {
		4
	};
}

impl<'a> Context<'a> {
	pub const MAX_LEN: usize = Self::max_len();
	pub const MIN_LEN: usize = Self::min_len();

	/// Panics if `value` is longer than [`Self::MAX_LEN`] or is 0.
	pub const fn from_bytes(value: &'a [u8]) -> Self {
		match Self::try_from_bytes(value) {
			Ok(ctx) => ctx,
			Err(err) => panic!("{}", err.const_display()),
		}
	}

	pub const fn try_from_bytes(value: &'a [u8]) -> Result<Self, ContextError> {
		let len = value.len();
		if len < Self::MIN_LEN {
			Err(ContextError::SliceTooShort)
		} else if len > Self::MAX_LEN {
			Err(ContextError::SliceTooLong(len))
		} else {
			Ok(Context(value))
		}
	}

	const fn max_len() -> usize {
		let result = ed25519_dalek::Context::<ed25519_dalek::VerifyingKey>::MAX_LENGTH;
		assert!(result == ctx_len!("max"));
		result
	}

	const fn min_len() -> usize {
		let result = ctx_len!("min");
		assert!(result <= Self::MAX_LEN);
		result
	}
}

impl<'a> TryFrom<&'a [u8]> for Context<'a> {
	type Error = ContextError;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
		Self::try_from_bytes(value)
	}
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum ContextError {
	#[error(
		"requires a slice of at least length {} but got a slice of length {0}",
		Context::MIN_LEN
	)]
	SliceTooShort,
	#[error(
		"requires a slice of at most length {} but got a slice of length {0}",
		Context::MAX_LEN
	)]
	SliceTooLong(usize),
}

impl ContextError {
	const fn const_display(&self) -> &str {
		match self {
			Self::SliceTooShort => {
				concat!("requires a slice of at least length ", ctx_len!("min"))
			}
			Self::SliceTooLong(_len) => {
				concat!("requires a slice of at most length ", ctx_len!("max"))
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_ctx_try_from() {
		let valid = [
			[0; Context::MIN_LEN].as_slice(),
			[0; Context::MAX_LEN].as_slice(),
		];
		for s in valid {
			assert_eq!(Context::from_bytes(s), Context::try_from_bytes(s).unwrap());
		}

		let too_short = [&[], [0; Context::MIN_LEN - 1].as_slice()];
		for s in too_short {
			assert_eq!(Context::try_from_bytes(s), Err(ContextError::SliceTooShort));
			assert!(std::panic::catch_unwind(|| Context::from_bytes(s)).is_err());
		}

		let too_long = [
			[0; Context::MAX_LEN + 1].as_slice(),
			[0; Context::MAX_LEN + 2].as_slice(),
		];
		for s in too_long {
			assert_eq!(
				Context::try_from_bytes(s),
				Err(ContextError::SliceTooLong(s.len()))
			);
			assert!(std::panic::catch_unwind(|| Context::from_bytes(s)).is_err());
		}
	}
}
