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

use eyre::{bail, Result, WrapErr};
use futures::{SinkExt, StreamExt};
use replicate_common::{
	data_model::{entity::EntityId, DataModel, LocalChanges, RemoteChanges, State},
	ClientId,
};
use url::Url;

use crate::{connect_to_url, Ascii};

use replicate_common::messages::instance::{Clientbound as Cb, Serverbound as Sb};
type RpcFramed = replicate_common::Framed<wtransport::stream::BiStream, Cb, Sb>;

/// Client api for interacting with a particular instance on the server.
/// Instances manage persistent, realtime state updates for many concurrent clients.
#[derive(Debug)]
pub struct Instance {
	_conn: wtransport::Connection,
	_url: Url,
	/// Used to reliably push state updates from server to client. This happens for all
	/// entities when the client initially connects, as well as when the server is
	/// marking an entity as "stable", meaning its state is no longer changing frame to
	/// frame. This allows the server to reduce network bandwidth.
	// _stable_states: RecvStream,
	/// Used for general RPC.
	_rpc: RpcFramed,
	/// Current sequence number.
	// TODO: Figure out how sequence numbers work
	_state_seq: StateSeq,
	dm: DataModel,
}

impl Instance {
	/// # Arguments
	/// - `url`: Url of the manager api. For example, `https://foobar.com/my/manager`
	///   or `192.168.1.1:1337/uwu/some_manager`.
	/// - `bearer_token`: optional, must be ascii otherwise we will panic.
	pub async fn connect(url: Url, bearer_token: Option<&str>) -> Result<Self> {
		let bearer_token = bearer_token.map(|s| {
			// Technically, bearer tokens only permit a *subset* of ascii. But I
			// didn't care enough to be that precise.
			Ascii::try_from(s).expect("to be in-spec, bearer tokens must be ascii")
		});
		let conn = connect_to_url(&url, bearer_token)
			.await
			.wrap_err("failed to connect to server")?;

		let bi = wtransport::stream::BiStream::join(
			conn.open_bi()
				.await
				.wrap_err("could not initiate bi stream")?
				.await
				.wrap_err("could not finish opening bi stream")?,
		);
		let mut rpc = RpcFramed::new(bi);

		// Do handshake before anything else
		let local_namespace = {
			rpc.send(Sb::HandshakeRequest(ClientId::random()))
				.await
				.wrap_err("failed to send handshake request")?;
			let Some(msg) = rpc.next().await else {
				bail!("Server disconnected before completing handshake");
			};
			let msg = msg.wrap_err("error while receiving handshake response")?;
			let Cb::HandshakeResponse(local_namespace) = msg else {
				bail!("invalid message during handshake");
			};
			local_namespace
		};

		Ok(Self {
			_conn: conn,
			_url: url,
			_state_seq: Default::default(),
			dm: DataModel::new(local_namespace),
			_rpc: rpc,
		})
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

/// The results of a state update pushed by the server.
// TODO: This is gonna go away now.
pub enum RecvState<'a> {
	DeletedEntities(&'a [EntityId]),
	StateUpdates {
		entities: &'a [EntityId],
		states: &'a [State],
	},
}

/// Sequence number for state messages
#[derive(Debug, Default)]
pub struct StateSeq;
