//! Client api for realtime state synchronization.
//!
//! The design of this is heavily inspired by [this article by GafferOnGames][gaffer].
//! It is a good idea to read it before diving into the code.
//!
//! # What is the state?
//! The state is represented as a number of entities each with an ID and associated
//! binary blob of state. Clients can claim authority over the state of entities,
//! updating the server with the entity's latest state. The server is responsible
//! for aggregating all of these updates into a unified picture, and sending those
//! updates back to the client.
//!
//! Because the state is represented as an arbitrary binary blob, the server is general
//! enough for use by many different applications. This means that it is not specific
//! to social vr, it could be used my many different applications. For example, one
//! could build a shared networked physics demo, and use this same server.
//!
//! # Authority
//! Authority means "who has authority to dictate the state of an entity". The server
//! arbitrates authority, and generally allows clients to claim authority over an
//! entity without interference. In the event that two clients claim authority over the
//! same entity and provide a state update, the server will resolve the conflict and
//! pick a winner.
//!
//! The server may also enforce certain rules on who can gain authority of an entity.
//! Each entity can have an allowlist or a denylist of users that can claim authority
//! over it. We will implement the ability to dynamically set this via an RPC API later.
//!
//! # Clients
//! Clients are identified by a unique identifier that should be the same across
//! disconnects and reconnects. The client must prove ownership of this unique
//! identifier.
//!
//! The most logical implementation strategy to accomplish this is to solve it the same
//! way that the nexus protocol does - every user is identified by a
//! [Decentralized Identifier][DID][^1], and they sign a message with the DID's associated
//! private key, proving that they are who they say they are. This is done via the
//! [`AuthenticationAttestation`] argument when connecting to the instance.
//!
//! # Entities
//! Each entity has state, and the instance has many entities. An entity is identified
//! with an id, which the server assigns in response to a request to spawn an entity
//! by the client. The id is used by the client to reference it.
//!
//! [^1]: For anyone concerned if this involves blockchains: There are several DID
//! methods that don't require any use of blockchain technologies, such as
//! [did:key][did:key] and [did:web][did:web]. Initially, we intend to support only
//! these two DID methods.
//!
//! [gaffer]: https://gafferongames.com/post/networked_physics_in_virtual_reality/
//! [DID]: https://www.w3.org/TR/did-core/
//! [did:key]: https://w3c-ccg.github.io/did-method-key/
//! [did:web]: https://w3c-ccg.github.io/did-method-web/

use std::{num::NonZeroU16, sync::atomic::AtomicU16};

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use replicate_common::did::AuthenticationAttestation;
use tracing::warn;
use url::Url;
use wtransport::{endpoint::ConnectOptions, ClientConfig, Endpoint, RecvStream};

use crate::CertHashDecodeErr;

/// The state of an [`Entity`].
pub type State = bytes::Bytes;

/// Client api for interacting with a particular instance on the server.
/// Instances manage persistent, realtime state updates for many concurrent clients.
#[derive(Debug)]
pub struct Instance {
	_conn: wtransport::Connection,
	_url: String,
	/// Used to reliably push state updates from server to client. This happens for all
	/// entities when the client initially connects, as well as when the server is
	/// marking an entity as "stable", meaning its state is no longer changing frame to
	/// frame. This allows the server to reduce network bandwidth.
	_stable_states: RecvStream,
	/// Current sequence number.
	// TODO: Figure out how sequence numbers work
	_state_seq: StateSeq,
}

impl Instance {
	/// # Arguments
	/// - `url`: Url of the manager api. For example, `https://foobar.com/my/manager`
	///   or `192.168.1.1:1337/uwu/some_manager`.
	/// - `auth_attest`: Used to provide to the server proof of our identity, based on
	///   our DID.
	pub async fn connect(
		url: Url,
		auth_attest: AuthenticationAttestation,
	) -> Result<Self, ConnectErr> {
		let _conn = connect_to_url(url, auth_attest).await?;
		todo!()
	}

	/// Asks the server to reserve for this client a list of entity ids and store them
	/// in `entities`.
	///
	/// Entities are not spawned on the server until their state is set.
	pub async fn reserve_entities(
		&self,
		#[allow(clippy::ptr_arg)] _entities: &mut Vec<Entity>,
	) -> Result<(), ReserveErr> {
		todo!()
	}

	pub async fn delete_entities(
		&self,
		_entities: impl IntoIterator<Item = Entity>,
	) -> Result<(), DeleteErr> {
		todo!()
	}

	/// Updates the state of an entity. Note that delivery of this state is not
	/// guaranteed.
	///
	/// If you want to gurantee delivery, use [`Self::send_reliable_state`] or
	/// use events or an RPC system.
	pub fn send_state(
		_states: impl IntoIterator<Item = (Entity, State)>,
	) -> Result<(), SendStateErr> {
		todo!()
	}

	/// Sends a reliable state update. Typically you only do this when you don't expect
	/// to modify the state for the forseeable future and care about the final result.
	/// For example, you can use this when a physics object comes to rest.
	///
	/// Dont use this for general purpose reliable delivery, use events or an RPC system
	/// for that.
	pub async fn send_reliable_state(
		_states: impl IntoIterator<Item = (Entity, State)>,
	) -> Result<(), SendReliableStateErr> {
		todo!()
	}

	// TODO: Handle receiving apis.
}

/// An identifier for an entity in the network datamodel. NOTE: This is not the
/// same as an ECS entity. This crate is completely independent of bevy.
///
/// Entity ID of 1 is guaranteed to never be assigned by the server, so you can
/// use it for default initialization
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Entity(NonZeroU16);

/// Sequence number for state messages
#[derive(Debug)]
pub struct StateSeq(AtomicU16);

mod error {
	use crate::CertHashDecodeErr;
	use wtransport::error::{ConnectingError, SendDatagramError, StreamWriteError};

	#[derive(thiserror::Error, Debug)]
	pub enum ReserveErr {}

	#[derive(thiserror::Error, Debug)]
	pub enum DeleteErr {}

	#[derive(thiserror::Error, Debug)]
	pub enum SendStateErr {
		#[error("error while sending state across network: {0}")]
		Dgram(#[from] SendDatagramError),
	}

	#[derive(thiserror::Error, Debug)]
	pub enum SendReliableStateErr {
		#[error("error while finalizing state to network: {0}")]
		StreamWrite(#[from] StreamWriteError),
	}

	#[derive(thiserror::Error, Debug)]
	pub enum ConnectErr {
		#[error("failed to create webtransport client: {0}")]
		ClientCreate(#[from] std::io::Error),
		#[error("failed to connect to webtransport endoint: {0}")]
		WtConnectingError(#[from] ConnectingError),
		#[error(transparent)]
		InvalidCertHash(#[from] CertHashDecodeErr),
		#[error(transparent)]
		Other(#[from] Box<dyn std::error::Error>),
	}
}
pub use self::error::*;

async fn connect_to_url(
	url: Url,
	auth_attest: AuthenticationAttestation,
) -> Result<wtransport::Connection, ConnectErr> {
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
	let opts = ConnectOptions::builder(&url)
		.add_header("Authorization", format!("Bearer {}", auth_attest))
		.build();
	Ok(client.connect(opts).await?)
}
