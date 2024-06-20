use std::ops::{Deref, DerefMut};

use bevy::{
	app::{Plugin, PostUpdate, PreUpdate, Update},
	ecs::{
		component::Component,
		entity::Entity,
		event::{Event, EventReader, EventWriter},
		query::{Added, With, Without},
		system::{CommandQueue, Commands, Query, Res, ResMut, Resource},
		world::World,
	},
	log::{error, trace},
	tasks::IoTaskPool,
	transform::components::{GlobalTransform, Transform},
};
use color_eyre::eyre::{Result, WrapErr as _};
use replicate_client::common::data_model::{DataModel, Entity as DmEntity};
use tokio::sync::mpsc;

const BOUNDED_CHAN_COMMAND_QUEUE_SIZE: usize = 16;

#[derive(Debug, Default)]
pub struct NetcodePlugin {}

impl Plugin for NetcodePlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_event::<ConnectToManagerRequest>()
			.add_event::<ConnectToManagerResponse>()
			.init_resource::<CommandQueueChannel>()
			.init_resource::<NetcodeDataModel>()
			.add_systems(PreUpdate, (apply_queued_commands, from_data_model))
			.add_systems(PostUpdate, (spawn_entities, to_data_model))
			.add_systems(Update, on_connect_to_manager_evt);
	}
}

#[derive(Debug, Resource)]
struct NetcodeManager(#[allow(unused)] replicate_client::manager::Manager);

/// Convenient way to receive commands sent from the async tasks.
#[derive(Debug, Resource)]
struct CommandQueueChannel {
	tx: mpsc::Sender<CommandQueue>,
	rx: mpsc::Receiver<CommandQueue>,
}

impl Default for CommandQueueChannel {
	fn default() -> Self {
		let (tx, rx) = mpsc::channel(BOUNDED_CHAN_COMMAND_QUEUE_SIZE);
		Self { tx, rx }
	}
}

fn apply_queued_commands(
	mut commands: Commands,
	mut chan: ResMut<CommandQueueChannel>,
) {
	while let Ok(mut command_queue) = chan.rx.try_recv() {
		commands.append(&mut command_queue)
	}
}

/// Other plugins create this when they want to connect to a manager.
#[derive(Debug, Event, Eq, PartialEq)]
pub struct ConnectToManagerRequest {
	/// The URL of the manager to connect to. If `None`, locally host.
	pub manager_url: Option<replicate_client::url::Url>,
}

/// Produced in response to [`ConnectToManagerRequest`].
#[derive(Debug, Event)]
pub struct ConnectToManagerResponse(pub Result<()>);

fn on_connect_to_manager_evt(
	command_queue: Res<CommandQueueChannel>,
	mut request: EventReader<ConnectToManagerRequest>,
	mut response: EventWriter<ConnectToManagerResponse>,
) {
	for ConnectToManagerRequest { manager_url } in request.read() {
		let Some(manager_url) = manager_url else {
			response.send(ConnectToManagerResponse(Ok(())));
			continue;
		};
		let pool = IoTaskPool::get();
		let manager_url = manager_url.to_owned();
		let tx = command_queue.tx.clone();
		pool.spawn(async_compat::Compat::new(async move {
			let connect_result =
				replicate_client::manager::Manager::connect(manager_url, None)
					.await
					.wrap_err("failed to connect to manager server");
			if let Err(ref err) = connect_result {
				error!("{err:?}");
			}

			// We use a command queue to enqueue commands back to bevy from the
			// async code.
			let mut queue = CommandQueue::default();
			let response_event = ConnectToManagerResponse(connect_result.map(|mngr| {
				queue.push(|w: &mut World| w.insert_resource(NetcodeManager(mngr)));
			}));
			queue.push(|w: &mut World| {
				w.send_event(response_event).expect("failed to send event");
			});
			match tx.send(queue).await {
				Ok(()) | Err(mpsc::error::SendError(_)) => (),
			}
		}))
		// We don't need to explicitly retrieve the return value.
		.detach();
	}
}

/// Add this to entities that should be synchronized over the network
#[derive(Debug, Eq, PartialEq, Component)]
pub struct Synchronized(pub DmEntity);

/// Add to entities that we claim to have authority over. Entities that are
/// `Synchronized` but don't have this component are entities that we do not
/// have authority over.
///
/// Note that according to its ownership rules, the data model may remove this
/// component and start overwriting the state in the data model, indicating that
/// remote peers have authority.
#[derive(Debug, Component)]
pub struct LocalAuthority;

#[derive(Debug, Default, Resource)]
pub struct NetcodeDataModel {
	dm: DmEnum,
}

impl Deref for NetcodeDataModel {
	type Target = DataModel;

	fn deref(&self) -> &Self::Target {
		&self.dm
	}
}

impl DerefMut for NetcodeDataModel {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.dm
	}
}

#[derive(Debug)]
pub enum DmEnum {
	#[allow(dead_code)]
	Remote(replicate_client::instance::Instance),
	Local(DataModel),
}

impl Deref for DmEnum {
	type Target = DataModel;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Remote(instance) => instance.data_model(),
			Self::Local(dm) => dm,
		}
	}
}

impl DerefMut for DmEnum {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Remote(instance) => instance.data_model_mut(),
			Self::Local(dm) => dm,
		}
	}
}

impl Default for DmEnum {
	fn default() -> Self {
		Self::Local(DataModel::default())
	}
}

// TODO: we should have some sort of state extractor trait that the netcode plugin can
// use instead of hard coding this
fn to_data_model(
	mut dm: ResMut<NetcodeDataModel>,
	query: Query<(&GlobalTransform, &Synchronized), With<LocalAuthority>>,
) {
	for (trans, sync) in query.iter() {
		trace!(entity=?sync.0, "wrote state");
		let serialized = serialize_transform(&trans.compute_transform());
		dm.dm
			.update(sync.0, serialized.into())
			.expect("todo: figure out what to do when server despawns entities")
	}
}

fn spawn_entities(
	mut dm: ResMut<NetcodeDataModel>,
	query: Query<
		(Entity, &GlobalTransform),
		(Added<LocalAuthority>, Without<Synchronized>),
	>,
	mut commands: Commands,
) {
	for (entity, trans) in query.iter() {
		trace!(bevy_entity=?entity, "spawning DataModel entity");
		let dm_entity =
			dm.spawn(serialize_transform(&trans.compute_transform()).into());

		commands.entity(entity).insert(Synchronized(dm_entity));
	}
}

fn from_data_model(
	dm: Res<NetcodeDataModel>,
	mut query: Query<(&mut Transform, &Synchronized), Without<LocalAuthority>>,
) {
	for (mut trans, sync) in query.iter_mut() {
		let serialized = dm
			.get(sync.0)
			.expect("todo: figure out what to do when server despawns entities");
		*trans = deserialize_transform(serialized);
	}
}

fn serialize_transform(trans: &Transform) -> Vec<u8> {
	serde_json::ser::to_vec(trans).expect("serialization should always succeed")
}

fn deserialize_transform(bytes: &[u8]) -> Transform {
	serde_json::from_slice(bytes).expect("todo: handle deserialization failure")
}
