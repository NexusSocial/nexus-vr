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
//! Internally, entity ids are based on a generational index, which is a tuple of
//! (generation, index): (u32, u32). In the future we may make this (u16,u16). This is
//! because using generations avoids the [ABA problem][ABA] where an entity could be
//! deleted, and then a stale id for it could be held by either the server or client
//! still, and then a new, distinct entity is later created at that index and accessed
//! with the stale id.
//!
//! # State Priority
//! Each connection will have a dynamically calculated bandwidth, measured in bytes per
//! second. Sending data too fast will cause state updates to be dropped. Because state
//! updates are supposed to happen at a roughly fixed frequency, we often will need to
//! proactively limit the amount of data being sent. To solve this, the user of the api
//! will explicitly set a send and recv proirity for each entity.
//!
//! On each message sent, the client will take the entity's priority and add it to a
//! priority ccumulator associated with that entity. Then, it will sort all entities by
//! their priority accumulators, adding entities with the highest proirity to the packet
//! until the packet has reached its target size (the one calculated based on bandwidth)
//! or there are no more entities with a non-negative priority accumulator. When the
//! entity is included in the packet, its priority accumulator is reset.
//!
//! When the client configures the recv priority, it will send a reliable message to
//! the server informing it of its recv priority for that entity. The server will track
//! these priorities for each client independently, and will follow a similar priority
//! accumulation scheme described above to prioritize sending updates to the client.
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
//! [ABA]: https://en.wikipedia.org/wiki/ABA_problem

use std::{
	sync::{atomic::AtomicU16, Arc, Mutex},
	time::Duration,
};

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use bytes::{Bytes, BytesMut};
use eyre::{bail, ensure, Result, WrapErr};
use futures::{Sink, SinkExt, StreamExt, TryStream};
use replicate_common::{
	data_model::{DataModel, LocalChanges, RemoteChanges},
	did::AuthenticationAttestation,
};
use tokio::{
	io::{AsyncRead, AsyncWrite},
	select,
	sync::oneshot,
};
use tracing::{info, warn};
use url::Url;
use wtransport::{endpoint::ConnectOptions, ClientConfig, Endpoint};

use crate::{unreliable::BiUnreliable, CertHashDecodeErr};

use replicate_common::messages::instance::{
	ClientboundReliable as CbR, ClientboundUnreliable as CbU, Mutations,
	ServerboundReliable as SbR, ServerboundUnreliable as SbU,
};

/// "Reliable Framed"
type RFramed<T = wtransport::stream::BiStream> = replicate_common::Framed<T, CbR, SbR>;
/// "Unreliable Framed"
type UFramed<T> =
	tokio_serde::Framed<T, CbU, SbU, tokio_serde::formats::Json<CbU, SbU>>;

/// Client api for interacting with a particular instance on the server.
/// Instances manage persistent, realtime state updates for many concurrent clients.
///
/// Other than the initial connect function, the `Instance` doesn't use async and is nonblocking,
/// so that it can be used inside synchronous code, such as a bevy system.
#[derive(Debug)]
pub struct Instance {
	dm: DataModel,
	_local: Arc<Mutex<LocalChanges>>,
	_remote: Arc<Mutex<RemoteChanges>>,
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
	) -> Result<(Self, NetTaskHandle)> {
		let conn = connect_to_url(&url, auth_attest)
			.await
			.wrap_err("failed to connect to server")?;

		let reliable = wtransport::stream::BiStream::join(
			conn.open_bi()
				.await
				.wrap_err("could not initiate bi stream")?
				.await
				.wrap_err("could not finish opening bi stream")?,
		);
		let unreliable = BiUnreliable::new(conn);

		let local = Arc::new(Mutex::new(LocalChanges::default()));
		let remote = Arc::new(Mutex::new(RemoteChanges::default()));
		let net_task =
			NetTaskHandle::spawn(local.clone(), remote.clone(), reliable, unreliable);
		let self_ = Self {
			dm: DataModel::new(),
			_local: local,
			_remote: remote,
		};
		Ok((self_, net_task))
	}

	/// Accesses the data model read-only.
	pub fn data_model(&self) -> &DataModel {
		&self.dm
	}

	/// Accesses the data model read-write.
	pub fn data_model_mut(&mut self) -> &mut DataModel {
		&mut self.dm
	}

	// This retrieves the [`RemoteChanges`] from the networking task, calls [`DataModel::flush`]` to apply the changes, and gives the [`LocalChanges`] to the networking task so that it can asynchronously update the server.
	pub fn flush_pending_changes(&mut self) {
		// TODO: Actually get these from a networking task.
		let remote_changes = RemoteChanges::default();
		let mut local_changes = LocalChanges::default();
		self.dm.flush(&remote_changes, &mut local_changes);

		// TODO: Actually send the local changes to the networking task.
	}
}

/// Sequence number for state messages
#[derive(Debug, Default)]
pub struct StateSeq(AtomicU16);

async fn connect_to_url(
	url: &Url,
	auth_attest: AuthenticationAttestation,
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
	let opts = ConnectOptions::builder(url)
		.add_header("Authorization", format!("Bearer {}", auth_attest))
		.build();
	Ok(client.connect(opts).await?)
}

/// Provides access to the networking task. You can join on errors and also kill it using this.
#[derive(Debug)]
pub struct NetTaskHandle {
	pub stop_tx: oneshot::Sender<()>,
	pub handle: tokio::task::JoinHandle<Result<()>>,
}

impl NetTaskHandle {
	pub fn spawn<R, U>(
		local: Arc<Mutex<LocalChanges>>,
		remote: Arc<Mutex<RemoteChanges>>,
		rpc_bi_transport: R,
		unreliable_updates: U,
	) -> Self
	where
		R: AsyncRead + AsyncWrite + Send + Unpin + 'static,
		U: TryStream<Ok = BytesMut, Error = eyre::Report>
			+ Sink<Bytes, Error = eyre::Report>
			+ Send
			+ Unpin
			+ 'static,
	{
		let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
		let handle = tokio::task::spawn(net_task(
			stop_rx,
			local,
			remote,
			rpc_bi_transport,
			unreliable_updates,
		));
		NetTaskHandle { stop_tx, handle }
	}
}

/// The entry point of the networking task.
/// # Arguments
async fn net_task<R, U>(
	mut stop_rx: oneshot::Receiver<()>,
	_local: Arc<Mutex<LocalChanges>>,
	_remote: Arc<Mutex<RemoteChanges>>,
	rpc_transport: R,
	unreliable_updates: U,
) -> Result<()>
where
	R: StreamTransport,
	U: MessageTransport,
{
	let mut rf = RFramed::new(rpc_transport);
	let json_codec: tokio_serde::formats::Json<CbU, SbU> = Default::default();
	let mut uf: UFramed<U> = tokio_serde::Framed::new(unreliable_updates, json_codec);

	// Do handshake before anything else
	{
		rf.send(SbR::HandshakeRequest)
			.await
			.wrap_err("failed to send handshake request")?;
		let Some(msg) = rf.next().await else {
			bail!("Server disconnected before completing handshake");
		};
		let msg = msg.wrap_err("error while receiving handshake response")?;
		ensure!(
			msg == CbR::HandshakeResponse,
			"invalid message during handshake"
		);
	}

	const SYNC_PERIOD: Duration = Duration::from_millis(100);
	let mut sync_interval = tokio::time::interval(SYNC_PERIOD);

	loop {
		select! {
			_ = &mut stop_rx => break,
			_ = sync_interval.tick() => push_unreliable(&mut uf).await.wrap_err("error in on_sync")?,
		}
	}
	info!("Stopping networking task...");

	Ok(())
}

async fn push_unreliable<U>(uf: &mut UFramed<U>) -> Result<()>
where
	U: TryStream<Ok = BytesMut, Error = eyre::Report>
		+ Sink<Bytes, Error = eyre::Report>
		+ Unpin,
{
	let cb: CbU = uf
		.next()
		.await
		.unwrap()
		.wrap_err("failed to receive ClientboundUnreliable")?;
	println!("received {cb:?}");
	let payload = SbU::Mutations(Mutations::default());
	uf.send(payload)
		.await
		.wrap_err("failed to send ServerboundUnreliable")?;
	Ok(())
}

/// Trait alias for a message-oriented bidirectional transport.
/// The items are still raw byte buffers.
trait MessageTransport:
	TryStream<Ok = BytesMut, Error = eyre::Report>
	+ Sink<Bytes, Error = eyre::Report>
	+ Unpin
{
}
impl<T> MessageTransport for T where
	T: TryStream<Ok = BytesMut, Error = eyre::Report>
		+ Sink<Bytes, Error = eyre::Report>
		+ Unpin
{
}

/// Trait alias for a byte-oriented bidirectional transport.
trait StreamTransport: AsyncRead + AsyncWrite + Unpin {}
impl<T> StreamTransport for T where T: AsyncRead + AsyncWrite + Unpin {}
