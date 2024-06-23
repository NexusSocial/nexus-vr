//! Instances are not purely client or server authoritative. Instead, they follow
//! a "distributed simulation with authority" scheme. This is inspired by an excellent
//! [article][gaffer_vr] on networked physics in VR by GafferOnGames.
//!
//! [gaffer_vr]: https://gafferongames.com/post/networked_physics_in_virtual_reality

pub mod manager;

use std::fmt::Debug;

use eyre::{bail, ensure, Result, WrapErr as _};
use futures::{SinkExt as _, StreamExt as _};
use replicate_common::{
	data_model::entity::Namespace,
	messages::instance::{Clientbound as Cb, Serverbound as Sb},
};
use tracing::{info, instrument};
use wtransport::endpoint::SessionRequest;

pub const URL_PREFIX: &str = "instances";

type Framed = replicate_common::Framed<wtransport::stream::BiStream, Sb, Cb>;

#[derive(Debug, Default)]
pub struct Instance {}

#[derive(derive_more::Deref, derive_more::DerefMut)]
struct Rng(pub Box<dyn rand::RngCore + Send>);

impl Debug for Rng {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.finish_non_exhaustive()
	}
}

#[derive(Debug)]
pub struct InstanceCtx {
	rng: Rng,
}

impl InstanceCtx {
	pub fn new(rng: impl rand::Rng + Sized + Send + 'static) -> Self {
		Self {
			rng: Rng(Box::new(rng)),
		}
	}
}

fn _assert_send() {
	fn helper(_: impl Send) {}
	helper(InstanceCtx::new(rand::rngs::OsRng));
}

#[instrument(skip_all, name = "instance")]
pub async fn handle_connection(
	mut ctx: InstanceCtx,
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
	{
		let Some(msg) = framed.next().await else {
			bail!("Client disconnected before completing handshake");
		};
		let msg = msg.wrap_err("error while receiving handshake message")?;
		ensure!(
			msg == Sb::HandshakeRequest,
			"invalid message during handshake"
		);
		framed
			.send(Cb::HandshakeResponse(Namespace(ctx.rng.next_u64())))
			.await
			.wrap_err("failed to send handshake response")?;
	}

	while let Some(request) = framed.next().await {
		let request: Sb = request.wrap_err("error while receiving message")?;
		match request {
			Sb::HandshakeRequest => {
				bail!("already did handshake, another handshake is unexpected")
			}
			_ => todo!(),
		}
	}

	info!("Client disconnected");
	Ok(())
}
