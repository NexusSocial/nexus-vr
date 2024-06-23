//! Instances are not purely client or server authoritative. Instead, they follow
//! a "distributed simulation with authority" scheme. This is inspired by an excellent
//! [article][gaffer_vr] on networked physics in VR by GafferOnGames.
//!
//! [gaffer_vr]: https://gafferongames.com/post/networked_physics_in_virtual_reality

pub mod manager;

use std::fmt::Debug;

use dashmap::DashMap;
use eyre::{bail, Result, WrapErr as _};
use futures::{SinkExt as _, StreamExt as _};
use replicate_common::{
	data_model::{
		entity::{EntityId, Namespace},
		State,
	},
	messages::instance::{Clientbound as Cb, Serverbound as Sb},
	ClientId,
};
use tracing::{info, instrument, trace};
use wtransport::endpoint::SessionRequest;

pub const URL_PREFIX: &str = "instances";

type Framed = replicate_common::Framed<wtransport::stream::BiStream, Sb, Cb>;

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("rejected client's attempt to claim authority")]
pub struct DeniedAuthorityErr;

/// Data stored for each entity
#[derive(Debug, Eq, PartialEq)]
struct EntityData {
	current_state: State,
	authority: ClientId,
}

#[derive(Debug, Default)]
pub struct DataModel {
	data: DashMap<EntityId, EntityData>,
}

impl DataModel {
	fn spawn(&self, entity: EntityId, state: State, authority: ClientId) {
		let insert_result = self.data.insert(
			entity,
			EntityData {
				current_state: state,
				authority,
			},
		);
		if let Some(already_exists) = insert_result {
			if already_exists.authority != authority {
				panic!("should have been impossible, we should only spawn on validated namespaces");
			}
			// The spawn event is out of date, so we ignore it
			trace!("stale spawn message");
		}
	}

	fn update(
		&self,
		entity: EntityId,
		state: State,
		authority: ClientId,
	) -> Result<(), DeniedAuthorityErr> {
		let insert_result = self.data.insert(
			entity,
			EntityData {
				current_state: state,
				authority,
			},
		);
		if insert_result.is_none() {
			trace!("missed a spawn message, spawning entity and updating state anyway.")
		}

		Ok(())
	}

	fn despawn(
		&self,
		entity: EntityId,
		_authority: ClientId,
	) -> Result<(), DeniedAuthorityErr> {
		let remove_result = self.data.remove(&entity);
		if remove_result.is_none() {
			trace!("tried to despawn a nonexistent entity, ignoring");
		}
		Ok(())
	}
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
struct Rng(pub Box<dyn rand::RngCore + Send + Sync>);

impl Debug for Rng {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.finish_non_exhaustive()
	}
}

#[derive(Debug)]
pub struct InstanceCtx {
	rng: Rng,
	dm: DataModel,
}

/// Information specific to a particular client, looked up via
pub struct ClientCtx {
	id: ClientId,
	namespace: Namespace,
}

impl InstanceCtx {
	pub fn new(rng: impl rand::Rng + Sized + Send + Sync + 'static) -> Self {
		Self {
			rng: Rng(Box::new(rng)),
			dm: DataModel::default(),
		}
	}
}

#[instrument(skip_all, name = "instance")]
pub async fn handle_connection(
	mut instance_ctx: InstanceCtx,
	session_request: SessionRequest,
) -> Result<()> {
	debug_assert!(session_request
		.path()
		.trim_start_matches('/')
		.starts_with(URL_PREFIX));
	let connection = session_request.accept().await?;
	info!("accepted wt connection to instance");
	let bi = wtransport::stream::BiStream::join(
		connection
			.accept_bi()
			.await
			.wrap_err("expected client to open bi stream")?,
	);
	let mut framed = Framed::new(bi);

	// Do handshake before anything else
	let client_ctx = {
		let Some(msg) = framed.next().await else {
			bail!("Client disconnected before completing handshake");
		};
		let Sb::HandshakeRequest(client_id) =
			msg.wrap_err("error while receiving handshake message")?
		else {
			bail!("invalid message during handshake")
		};
		let namespace = Namespace(instance_ctx.rng.next_u64());
		framed
			.send(Cb::HandshakeResponse(namespace))
			.await
			.wrap_err("failed to send handshake response")?;

		ClientCtx {
			id: client_id,
			namespace,
		}
	};

	let reliable_fut = async {
		while let Some(request) = framed.next().await {
			let request: Sb = request.wrap_err("error while receiving message")?;
			handle_state_update(&instance_ctx, &client_ctx, request)?;
		}
		info!("Client disconnected");
		Ok::<(), eyre::Report>(())
	};

	let unreliable_fut = async { Ok::<(), eyre::Report>(()) };

	let _: ((), ()) = tokio::try_join!(reliable_fut, unreliable_fut)?;

	Ok(())
}

fn handle_state_update(
	instance_ctx: &InstanceCtx,
	client_ctx: &ClientCtx,
	message: Sb,
) -> Result<()> {
	match message {
		Sb::HandshakeRequest(_) => {
			bail!("already did handshake, another handshake is unexpected");
		}
		Sb::SpawnEntities { states } => {
			for (idx, state) in states {
				instance_ctx.dm.spawn(
					EntityId {
						namespace: client_ctx.namespace,
						idx,
					},
					state,
					client_ctx.id,
				)
			}
		}
		Sb::UpdateEntities { namespace, states } => {
			for (idx, state) in states {
				// TODO: Decide how to handle authority problems.
				let _ = instance_ctx.dm.update(
					EntityId { namespace, idx },
					state,
					client_ctx.id,
				);
			}
		}
		Sb::DespawnEntities {
			namespace,
			entities,
		} => {
			for idx in entities {
				// TODO: Decide how to handle authority problems.
				let _ = instance_ctx
					.dm
					.despawn(EntityId { namespace, idx }, client_ctx.id);
			}
		}
	}
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	fn _assert_instance_ctx_send() {
		fn helper(_: impl Send) {}
		helper(InstanceCtx::new(rand::rngs::OsRng));
	}
}
