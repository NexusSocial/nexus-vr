//! Plugins facilitating avatars.

pub mod assign;
mod loading;

use bevy::{
	prelude::{
		default, Added, App, Bundle, Commands, Entity, Name, Plugin, Query,
		RemovedComponents, ResMut, Update,
	},
	transform::TransformBundle,
};

use self::{assign::AvatarSelectPlugin, loading::AvatarLoadPlugin};
use crate::controllers::KeyboardController;

use social_common::humanoid::HumanoidPlugin;
use social_networking::data_model::{self as dm, DmEntity};

/// Plugins for the [`avatars`](self) module.
#[derive(Default)]
pub struct AvatarsPlugin;

impl Plugin for AvatarsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((HumanoidPlugin, AvatarLoadPlugin, AvatarSelectPlugin));

		if app.is_plugin_added::<social_networking::ClientPlugin>() {
			app.add_systems(Update, local_avatar_map);
		}
	}
}

#[derive(Bundle, Debug)]
pub struct LocalAvatar {
	pub name: Name,
	pub transform: TransformBundle,
	pub keeb_controller: KeyboardController,
}

impl Default for LocalAvatar {
	fn default() -> Self {
		Self {
			name: Name::from("Local Avatar"),
			transform: default(),
			keeb_controller: default(),
		}
	}
}

/// Updates data model mapping.
fn local_avatar_map(
	mut cmds: Commands,
	added: Query<Entity, Added<KeyboardController>>,
	mut removed: RemovedComponents<KeyboardController>,
	mut dm_local: ResMut<dm::LocalAvatars>,
) {
	for added in added.iter() {
		let dm_entity = DmEntity(cmds.spawn_empty().id());
		dm_local.0.insert(dm_entity, added);
	}

	for removed in removed.read() {
		dm_local.0.remove(removed);
	}
}

// TODO: map dm::RemoteAvatars
