//! Handles management of instances.
//!
//! We use a functional design here. Instead of a struct to manage the instances, we
//! just have an async function which runs on its own tokio task and communicate with
//! it by sending messages (CSP style). See [`run`] for the task body, and see
//! [`Handle`] and [`HandleReceiver`] for the messaging.

use std::{num::NonZeroU32, time::Duration};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

/// Used by instance manager task to communicate with [`Handle`].
#[derive(Debug)]
pub struct HandleReceiver;

/// Used to communicate with the instance manager task. Cheaply cloneable.
#[derive(Clone, Debug)]
pub struct Handle;

impl Handle {
	pub async fn new_instance(&self) -> InstanceId {
		// TODO: Use a channel to communicate
		tokio::time::sleep(Duration::from_millis(500)).await;
		InstanceId::new(1)
	}
}

pub fn make_handle() -> (Handle, HandleReceiver) {
	(Handle, HandleReceiver)
}

/// Spawn a task for the instance manager, and provides a way to join on it and
/// communicate with it.
pub fn spawn() -> (Handle, JoinHandle<Result<()>>) {
	let (outer, inner) = make_handle();
	(outer, tokio::spawn(run(inner)))
}

/// Uniquely identifies an instance. IDs can eventually be reused, but over a very very
/// long time span.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstanceId(NonZeroU32);

impl InstanceId {
	#[allow(dead_code)]
	pub fn id(&self) -> u32 {
		self.0.into()
	}

	/// # Panics
	/// Panics if id is 0
	fn new(id: u32) -> Self {
		let id = NonZeroU32::new(id).expect("id must be non zero");
		Self(id)
	}
}

/// The body of the task.
async fn run(_handle: HandleReceiver) -> Result<()> {
	std::future::pending().await
}
