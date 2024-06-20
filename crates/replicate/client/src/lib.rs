use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};
use eyre::Result;
use tracing::warn;
use url::Url;

use wtransport::{endpoint::ConnectOptions, ClientConfig, Endpoint};

pub use replicate_common as common;
pub use url;

pub mod instance;
pub mod manager;

/// An error when decoding a certificate Hash
#[derive(thiserror::Error, Debug)]
pub enum CertHashDecodeErr {
	#[error("expected url-safe base64 encoded fragment")]
	InvalidBase64(#[from] base64::DecodeError),
	#[error("expected length of 32, got length of {0}")]
	InvalidLen(usize),
}

/// A string that has been validated to be ascii.
struct Ascii<'a>(&'a str);

impl<'a> TryFrom<&'a str> for Ascii<'a> {
	type Error = ();

	fn try_from(value: &'a str) -> std::prelude::v1::Result<Self, Self::Error> {
		if value.is_ascii() {
			Ok(Self(value))
		} else {
			Err(())
		}
	}
}

/// If there is a url fragment, it will be treated as a server certificate hash.
async fn connect_to_url(
	url: &Url,
	bearer_token: Option<Ascii<'_>>,
) -> Result<wtransport::Connection> {
	let cert_hash = if let Some(frag) = url.fragment() {
		let cert_hash = BASE64_URL_SAFE_NO_PAD
			.decode(frag)
			.map_err(CertHashDecodeErr::from)?;
		let len = cert_hash.len();
		let cert_hash: [u8; 32] = cert_hash
			.try_into()
			.map_err(|_| CertHashDecodeErr::InvalidLen(len))?;
		Some(cert_hash)
	} else {
		None
	};

	let cfg = ClientConfig::builder().with_bind_default();
	let cfg = if let Some(_cert_hash) = cert_hash {
		// TODO: Implement self signed certs properly:
		// https://github.com/BiagioFesta/wtransport/issues/128
		warn!(
			"`serverCertificateHashes` is not yet supported, turning off \
                cert validation."
		);
		cfg.with_no_cert_validation()
	} else {
		cfg.with_native_certs()
	}
	.build();

	let client = Endpoint::client(cfg)?;
	let opts = ConnectOptions::builder(url);
	let opts = if let Some(b) = bearer_token {
		opts.add_header("Authorization", format!("Bearer {}", b.0))
	} else {
		opts
	};
	Ok(client.connect(opts.build()).await?)
}
