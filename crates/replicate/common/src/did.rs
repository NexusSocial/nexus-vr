use std::fmt::Display;

use bytes::Bytes;

/// The private key portion of the user's [`Did`].
/// TODO: Make this use actual cryptography and Nexus.
pub struct DidPrivateKey;

/// The ed25519 signature of a message.
/// TODO: Use actual cryptography instead of no-op.
#[derive(Debug)]
pub struct Signature;

/// The Decentrailized Id, i.e. the account identifier.
/// TODO: Make this use actual cryptography and Nexus.
#[derive(Debug)]
pub struct Did(pub String);

/// A message signed by the user's account. If you have an instance of this struct,
/// you can be sure that the signature is valid.
#[derive(Debug)]
pub struct SignedMessage {
	did: Did,
	_sig: Signature,
	msg: Bytes,
}

impl SignedMessage {
	/// Creates a `SignedMessage`.
	/// # Panics
	/// Panics if the `did` and `private_key` don't match.
	pub fn sign(msg: Bytes, did: Did, _private_key: &DidPrivateKey) -> Self {
		// TODO: Actually do the cryptography
		Self::verify(msg, did, Signature).expect(
			"verification of generated signature failed, did private key match DID?",
		)
	}

	pub fn msg(&self) -> &[u8] {
		&self.msg
	}

	pub fn did(&self) -> &Did {
		&self.did
	}

	pub fn verify(msg: Bytes, did: Did, signature: Signature) -> Result<Self, String> {
		// TODO: Do actual cryptography by checking signature against message
		Ok(Self {
			did,
			_sig: signature,
			msg,
		})
	}
}

/// Provides evidence that the client actually owns the DID that they claim they own.
#[derive(Debug)]
pub struct AuthenticationAttestation(SignedMessage);

impl AuthenticationAttestation {
	pub fn new(did: Did, private_key: &DidPrivateKey) -> Self {
		Self(SignedMessage::sign(
			did.0.clone().into_bytes().into(),
			did,
			private_key,
		))
	}

	/// Promotes a message to an authentication attestation, by checking that the
	/// message is genuine.
	pub fn verify(
		msg: Bytes,
		sig: Signature,
	) -> Result<Self, Box<dyn std::error::Error>> {
		let did = std::str::from_utf8(&msg)?;
		let did = Did(did.to_owned());
		let signed = SignedMessage::verify(msg, did, sig)
			.map_err(|_| "failed to verify message signature")?;
		Ok(AuthenticationAttestation(signed))
	}
}

impl Display for AuthenticationAttestation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.did.0)
	}
}
