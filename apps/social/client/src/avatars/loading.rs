//! Behavior associated with the initial loading/spawning of avatars
//!
//! This behavior is initiated when a vrm is spawned. You can query for fully loaded
//! avatars via [`FullyLoadedAvatar`].

use bevy::{
	asset::AssetServer,
	prelude::{
		debug, Commands, Component, Entity, EventWriter, Handle, Plugin, Query, ResMut,
		Update, With, Without,
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

#[derive(Component, Reflect)]
pub struct RigRequestSent;

fn on_vrm_asset_load(
	assets: ResMut<AssetServer>,
	mut cmds: Commands,
	mut assign_rig_evts: EventWriter<AutoAssignRigRequest>,
	entities_w_handle: Query<(Entity, &Handle<Vrm>), Without<RigRequestSent>>,
) {
	for (entity, vrm_handle) in entities_w_handle.iter() {
		if assets.is_loaded_with_dependencies(vrm_handle) {
			cmds.entity(entity).insert(RigRequestSent);
			debug!("Requesting rig autoassignment for entity {entity:?}");
			assign_rig_evts.send(AutoAssignRigRequest { mesh: entity })
		}
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

#[cfg(test)]
mod test {
	use crate::MainPlugin;

	use bevy::prelude::App;

	#[test]
	fn test_vrm_load() {
		let mut app = App::new();
		app.add_plugins(MainPlugin {
			exec_mode: crate::ExecutionMode::Testing,
			server_addr: None,
		});
		app.run();
	}
}
