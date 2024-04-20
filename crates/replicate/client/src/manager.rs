//! Messages for the manager API.

use std::fmt::Debug;

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use eyre::{bail, ensure, eyre, Context};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use replicate_common::{
	did::AuthenticationAttestation,
	messages::manager::{Clientbound as Cb, Serverbound as Sb},
	InstanceId,
};
use tracing::warn;
use url::Url;
use wtransport::{endpoint::ConnectOptions, ClientConfig, Endpoint};

use crate::CertHashDecodeErr;

type Result<T> = eyre::Result<T>;
type Framed = replicate_common::Framed<wtransport::stream::BiStream, Cb, Sb>;

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
	framed: Framed,
}

impl Manager {
	/// # Arguments
	/// - `url`: Url of the manager api. For example, `https://foobar.com/my/manager`
	///   or `192.168.1.1:1337/uwu/some_manager`.
	/// - `auth_attest`: Used to provide to the server proof of our identity, based on
	///   our DID.
	pub async fn connect(
		url: Url,
		auth_attest: &AuthenticationAttestation,
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
		let conn = client
			.connect(opts)
			.await
			.wrap_err("failed to connect to server")?;
		let bi = wtransport::stream::BiStream::join(
			conn.open_bi()
				.await
				.wrap_err("could not initiate bi stream")?
				.await
				.wrap_err("could not finish opening bi stream")?,
		);

		let mut framed = Framed::new(bi);

		// Do handshake before anything else
		{
			framed
				.send(Sb::HandshakeRequest)
				.await
				.wrap_err("failed to send handshake request")?;
			let Some(msg) = framed.next().await else {
				bail!("Server disconnected before completing handshake");
			};
			let msg = msg.wrap_err("error while receiving handshake response")?;
			ensure!(
				msg == Cb::HandshakeResponse,
				"invalid message during handshake"
			);
		}

		Ok(Self {
			_conn: conn,
			_url: url,
			framed,
		})
	}

	pub async fn instance_create(&mut self) -> Result<InstanceId> {
		self.framed
			.send(Sb::InstanceCreateRequest)
			.await
			.wrap_err("failed to write message")?;
		match self.framed.next().await {
			None => Err(eyre!("server disconnected")),
			Some(Err(err)) => {
				Err(eyre::Report::new(err).wrap_err("failed to receive message"))
			}
			Some(Ok(Cb::InstanceCreateResponse { id })) => Ok(id),
			Some(Ok(_)) => Err(eyre!("unexpected response")),
		}
	}

	pub async fn instance_url(&mut self, id: InstanceId) -> Result<Url> {
		self.framed
			.send(Sb::InstanceUrlRequest { id })
			.await
			.wrap_err("failed to write message")?;
		match self.framed.next().await {
			None => Err(eyre!("server disconnected")),
			Some(Err(err)) => {
				Err(eyre::Report::new(err).wrap_err("failed to receive message"))
			}
			Some(Ok(Cb::InstanceUrlResponse { url })) => Ok(url),
			Some(Ok(_)) => Err(eyre!("unexpected response")),
		}
	}
}
