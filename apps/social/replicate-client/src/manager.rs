use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use replicate_common::{did::AuthenticationAttestation, InstanceId};
use tracing::warn;
use url::Url;
use wtransport::{
	endpoint::ConnectOptions,
	error::{
		ConnectingError, ConnectionError, StreamOpeningError, StreamReadExactError,
		StreamWriteError,
	},
	ClientConfig, Endpoint,
};

use crate::CertHashDecodeErr;

type Result<T> = std::result::Result<T, ManagerErr>;

/// Manages instances on the instance server. Under the hood, this is all done
/// as reliable messages on top of a WebTransport connection.
///
/// Note: Clients must have permission to manage the server in order to connect.
/// This is initially going to be implemented as an allowlist in the server of
/// user IDs.
#[derive(Debug)]
pub struct Manager {
	_conn: wtransport::Connection,
	_url: Url,
	sender: wtransport::SendStream,
	receiver: wtransport::RecvStream,
}

impl Manager {
	/// # Arguments
	/// - `url`: Url of the manager api. For example, `https://foobar.com/my/manager`
	///   or `192.168.1.1:1337/uwu/some_manager`.
	/// - `auth_attest`: Used to provide to the server proof of our identity, based on
	///   our DID.
	pub async fn connect(
		url: Url,
		auth_attest: AuthenticationAttestation,
	) -> Result<Self> {
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
			warn!(
				"`serverCertificateHashes` is not yet supported, turning off \
                cert validation."
			);
			// TODO: Use the cert hash as the root cert instead of no validation
			cfg.with_no_cert_validation()
		} else {
			cfg.with_native_certs()
		}
		.build();

		let client = Endpoint::client(cfg)?;
		let opts = ConnectOptions::builder(&url)
			.add_header("Authorization", format!("Bearer {}", auth_attest))
			.build();
		let conn = client.connect(opts).await?;
		let (sender, receiver) = conn.open_bi().await?.await?;
		Ok(Self {
			_conn: conn,
			_url: url,
			sender,
			receiver,
		})
	}

	pub async fn instance_create(&mut self) -> Result<InstanceId> {
		self.sender.write_all("create".as_bytes()).await?;
		const ACK: &str = "ack";
		let mut response = [0; ACK.len()];
		self.receiver.read_exact(&mut response).await?;
		if response != ACK.as_bytes() {
			return Err(ManagerErr::UnexpectedResponse);
		}

		todo!()
	}
}

#[derive(thiserror::Error, Debug)]
pub enum ManagerErr {
	#[error("failed to create webtransport client: {0}")]
	ClientCreate(#[from] std::io::Error),
	#[error("failed to connect to webtransport endoint: {0}")]
	WtConnectingError(#[from] ConnectingError),
	#[error(transparent)]
	InvalidCertHash(#[from] CertHashDecodeErr),
	#[error(transparent)]
	ConnectionError(#[from] ConnectionError),
	#[error(transparent)]
	StreamOpeningError(#[from] StreamOpeningError),
	#[error(transparent)]
	StreamWriteError(#[from] StreamWriteError),
	#[error("not enough bytes read on stream")]
	StreamReadExactError(#[from] StreamReadExactError),
	#[error("unexpected response")]
	UnexpectedResponse,
}
