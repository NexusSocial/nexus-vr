//! Plugins facilitating avatars.

pub mod assign;
mod loading;

use bevy::prelude::With;
use bevy::{
	prelude::{
		default, Added, App, BuildChildren, Bundle, Changed, Commands, Component,
		Entity, IntoSystemConfigs, Name, Plugin, PreUpdate, Query, RemovedComponents,
		Res, Transform, Update,
	},
	reflect::Reflect,
	transform::TransformBundle,
};

use self::{assign::AvatarSelectPlugin, loading::AvatarLoadPlugin};
use crate::controllers::KeyboardController;

use social_common::humanoid::HumanoidPlugin;
use social_networking::data_model::{self as dm, AvatarBundle};
use crate::custom_audio::spatial_audio::SpatialAudioListener;

/// Plugins for the [`avatars`](self) module.
#[derive(Default)]
pub struct AvatarsPlugin;

impl Plugin for AvatarsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((HumanoidPlugin, AvatarLoadPlugin, AvatarSelectPlugin))
			.register_type::<DmEntity>()
			.register_type::<LocalEntity>();

		if app.is_plugin_added::<social_networking::ClientPlugin>() {
			app.add_systems(PreUpdate, (added_dm_entity, removed_dm_entity))
				.add_systems(Update, (write_pose, read_pose).chain());
		}
	}
}

/// Adding this will inform this plugin to begin synchronizing between the entity this
/// component is attached to, and the entity in the data model (the one in the tuple).
#[derive(Component, Reflect, Debug, Copy, Clone)]
pub struct DmEntity(pub Entity);

#[derive(Component, Reflect, Debug, Copy, Clone)]
pub struct LocalEntity(pub Entity);

#[derive(Bundle, Debug)]
pub struct LocalAvatar {
	pub name: Name,
	pub transform: TransformBundle,
	pub keeb_controller: KeyboardController,
	pub spatial_audio_listener: SpatialAudioListener,
}

impl Default for LocalAvatar {
	fn default() -> Self {
		Self {
			name: Name::from("Local Avatar"),
			transform: default(),
			keeb_controller: default(),
			spatial_audio_listener: SpatialAudioListener,
		}
	}
}

/// Updates data model mapping.
fn added_dm_entity(
	mut cmds: Commands,
	added: Query<&DmEntity, Added<DmEntity>>,
	dm_root: Res<dm::DataModelRoot>,
) {
	for dm_entity in added.iter() {
		cmds.entity(dm_entity.0)
			.set_parent(dm_root.0)
			.insert(AvatarBundle::default());
	}
}

fn removed_dm_entity(
	_removed: RemovedComponents<DmEntity>,
	_dm_root: Res<dm::DataModelRoot>,
) {
	// TODO: despawn the entry in the data model?
}

fn write_pose(
	mut poses: Query<&mut dm::PlayerPose>,
	local_root_transforms: Query<
		(&Transform, &DmEntity),
		(Changed<Transform>, With<KeyboardController>),
	>,
) {
	for (t, &DmEntity(dm_entity)) in local_root_transforms.iter() {
		let mut pose = poses.get_mut(dm_entity).expect("no matching dm entity");
		pose.root.trans = t.translation;
		pose.root.rot = t.rotation;
	}
}

fn read_pose(
	mut local_root_transforms: Query<&mut Transform, With<DmEntity>>,
	poses: Query<(&dm::PlayerPose, &LocalEntity), Changed<dm::PlayerPose>>,
) {
	for (player_pose, &LocalEntity(local_entity)) in poses.iter() {
		let mut local_root_transform = local_root_transforms
			.get_mut(local_entity)
			.expect("no matching local entity");
		local_root_transform.translation = player_pose.root.trans;
		local_root_transform.rotation = player_pose.root.rot;
	}
}
