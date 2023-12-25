//! Plugins facilitating avatars.

pub mod loading;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use social_common::humanoid::HumanoidPlugin;

use self::loading::AvatarLoadPlugin;

/// Plugins for the [`avatars`](self) module.
#[derive(Default)]
pub struct Avatars;

impl PluginGroup for Avatars {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(HumanoidPlugin)
			.add(AvatarLoadPlugin)
	}
}
