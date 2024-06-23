use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};

/// Newtype on [`wtransport::Identity`].
pub(super) struct Certificate {
	pub(super) cert: wtransport::Identity,
	pub(super) base64_hash: String,
}

impl Certificate {
	pub fn new(cert: wtransport::Identity) -> Self {
		let cert_hash = cert
			.certificate_chain()
			.as_slice()
			.last()
			.expect("should be at least one cert")
			.hash();
		let base64_hash = BASE64_URL_SAFE_NO_PAD.encode(cert_hash.as_ref());
		Self { cert, base64_hash }
	}

	pub fn self_signed<I, S>(
		subject_alt_names: I,
	) -> Result<Self, wtransport::tls::error::InvalidSan>
	where
		I: Iterator<Item = S>,
		S: AsRef<str>,
	{
		wtransport::Identity::self_signed(subject_alt_names).map(Self::new)
	}
}

impl Clone for Certificate {
	fn clone(&self) -> Self {
		Self {
			cert: self.cert.clone_identity(),
			base64_hash: self.base64_hash.clone(),
		}
	}
}

impl From<wtransport::Identity> for Certificate {
	fn from(value: wtransport::Identity) -> Self {
		Self::new(value)
	}
}

impl std::fmt::Debug for Certificate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple(std::any::type_name::<Self>())
			.field(&self.cert.certificate_chain())
			.finish()
	}
}

impl std::fmt::Display for Certificate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_list()
			.entries(
				self.cert
					.certificate_chain()
					.as_slice()
					.iter()
					.map(|c| c.hash().fmt(wtransport::tls::Sha256DigestFmt::DottedHex)),
			)
			.finish()
	}
}

impl AsRef<wtransport::Identity> for Certificate {
	fn as_ref(&self) -> &wtransport::Identity {
		&self.cert
	}
}
