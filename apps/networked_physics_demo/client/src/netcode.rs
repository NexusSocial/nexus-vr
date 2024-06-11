use std::ops::{Deref, DerefMut};

use bevy::{
	app::{Plugin, PostUpdate, PreUpdate, Update},
	ecs::{
		component::Component,
		entity::Entity,
		event::{Event, EventReader},
		query::{Added, With, Without},
		schedule::NextState,
		system::{Commands, Query, Res, ResMut, Resource},
	},
	log::trace,
	reflect::Reflect,
	transform::components::{GlobalTransform, Transform},
};
use replicate_client::common::data_model::{DataModel, Entity as DmEntity};

use crate::GameModeState;

#[derive(Debug)]
pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<ConnectToManager>()
			.add_event::<ConnectToManager>()
			.init_resource::<NetcodeDataModel>()
			.add_systems(PreUpdate, from_data_model)
			.add_systems(PostUpdate, (spawn_entities, to_data_model))
			.add_systems(Update, on_connect_to_manager_evt);
	}
}

/// Other plugins create this when they want to connect to a manager.
#[derive(Debug, Reflect, Event, Eq, PartialEq)]
pub struct ConnectToManager {
	/// The URL of the manager to connect to
	pub manager_url: String,
}

fn on_connect_to_manager_evt(
	mut connect_to_manager: EventReader<ConnectToManager>,
	mut next_state: ResMut<NextState<GameModeState>>,
) {
	for ConnectToManager { manager_url: _ } in connect_to_manager.read() {
		// TODO: Actually connect to the manager instead of faking it
		next_state.set(GameModeState::InMinecraft);
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
