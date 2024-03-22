//! Behavior associated with the initial loading/spawning of avatars
//!
//! This behavior is initiated when a vrm is spawned. You can query for fully loaded
//! avatars via [`FullyLoadedAvatar`].

use bevy::{
	prelude::{
		debug, AssetEvent, Commands, Component, Entity, EventReader, EventWriter,
		Handle, Plugin, Query, Update, With, Without,
	},
	reflect::Reflect,
};
use bevy_vrm::Vrm;
use social_common::humanoid::{AutoAssignRigRequest, HumanoidRig};

pub struct AvatarLoadPlugin;

impl Plugin for AvatarLoadPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<FullyLoadedAvatar>()
			.add_systems(Update, on_vrm_asset_load)
			.add_systems(Update, check_for_fully_loaded);
	}
}

/// Inserted once the [`AvatarLoadPlugin`] has confirmed that the avatar is fully
/// loaded. You often should include this in your query when working on avatars.
#[derive(Component, Reflect)]
pub struct FullyLoadedAvatar;

fn on_vrm_asset_load(
	mut vrm_load_evts: EventReader<AssetEvent<Vrm>>,
	mut assign_rig_evts: EventWriter<AutoAssignRigRequest>,
	entities_w_handle: Query<(Entity, &Handle<Vrm>)>,
) {
	for evt in vrm_load_evts.read() {
		let AssetEvent::LoadedWithDependencies { id } = evt else {
			continue;
		};
		let Some(entity_matching_id) = entities_w_handle
			.iter()
			.find_map(|(e, h)| (h.id() == *id).then_some(e))
		else {
			continue;
		};

		debug!("Requesting rig autoassignment for entity {entity_matching_id:?}");
		assign_rig_evts.send(AutoAssignRigRequest {
			mesh: entity_matching_id,
		});
	}
}

fn check_for_fully_loaded(
	mut cmds: Commands,
	required_components: Query<Entity, (With<HumanoidRig>, Without<FullyLoadedAvatar>)>,
) {
	for e in required_components.iter() {
		cmds.entity(e).insert(FullyLoadedAvatar);
		debug!("Fully loaded entity: {e:?}");
	}
}
