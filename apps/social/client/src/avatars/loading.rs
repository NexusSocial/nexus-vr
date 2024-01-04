//! Behavior associated with the initial loading/spawning of avatars
//!
//! This behavior is initiated when a vrm is spawned. You can query for fully loaded
//! avatars via [`FullyLoadedAvatar`].

use bevy::prelude::{info, Name};
use bevy::{
	asset::AssetServer,
	prelude::{
		debug, Commands, Component, Entity, EventWriter, Handle, Plugin, Query, ResMut,
		Update, With, Without,
	},
	reflect::Reflect,
};
use bevy_vrm::Vrm;
use social_common::humanoid::{AutoAssignRigRequest, HumanoidPlugin, HumanoidRig};

pub struct AvatarLoadPlugin;

impl Plugin for AvatarLoadPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		assert!(app.is_plugin_added::<HumanoidPlugin>());
		app.register_type::<FullyLoadedAvatar>()
			.register_type::<RigRequestSent>()
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
	required_components: Query<
		(Entity, Option<&Name>),
		(With<HumanoidRig>, Without<FullyLoadedAvatar>),
	>,
) {
	for (e, name) in required_components.iter() {
		cmds.entity(e).insert(FullyLoadedAvatar);
		if let Some(name) = name {
			info!(
				"Fully loaded humanoid avatar entity: {:?} name: {}",
				e, name
			);
		} else {
			info!("Fully loaded entity: {e:?}");
		}
	}
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use bevy::app::ScheduleRunnerPlugin;
	use bevy::prelude::*;
	use bevy::winit::WinitPlugin;
	use bevy::{
		app::AppExit,
		prelude::{App, EventWriter, Query, Update},
	};
	use bevy_oxr::DefaultXrPlugins;
	use bevy_vrm::{VrmBundle, VrmPlugin};
	use social_common::humanoid::HumanoidPlugin;

	use super::{AvatarLoadPlugin, FullyLoadedAvatar};
	use crate::ASSET_FOLDER;

	#[test]
	fn test_vrm_load() {
		App::new()
			.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1) / 120))
			.add_plugins(
				DefaultXrPlugins::default()
					.build()
					.disable::<WinitPlugin>()
					.set(AssetPlugin {
						file_path: ASSET_FOLDER.to_string(),
						..Default::default()
					}),
			)
			.add_plugins(VrmPlugin)
			.add_plugins(HumanoidPlugin)
			.add_plugins(AvatarLoadPlugin)
			.add_systems(Startup, setup)
			.add_systems(Update, (on_fully_loaded, timeout_error))
			.run();

		#[derive(Resource)]
		struct VrmEntity(Entity);

		fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
			let vrm_entity = cmds
				.spawn(VrmBundle {
					vrm: asset_server
						.load(std::path::Path::new(ASSET_FOLDER).join("malek.vrm")),
					scene_bundle: SceneBundle { ..default() },
				})
				.id();
			cmds.insert_resource(VrmEntity(vrm_entity));
		}

		fn on_fully_loaded(
			fully_loaded: Query<Entity, With<FullyLoadedAvatar>>,
			vrm_entity: Res<VrmEntity>,
			mut exit_evts: EventWriter<AppExit>,
		) {
			if let Ok(fully_loaded) = fully_loaded.get_single() {
				assert!(
					vrm_entity.0 == fully_loaded,
					"fully loaded entity should be the vrm from setup"
				);
				exit_evts.send_default()
			}
		}

		fn timeout_error(time: Res<Time>) {
			assert!(time.elapsed() < Duration::from_secs(10), "test timed out")
		}
	}
}
