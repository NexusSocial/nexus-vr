pub use replicate_common as common;

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
