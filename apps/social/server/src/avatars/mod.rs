//! Plugins facilitating avatars.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use social_common::humanoid::HumanoidPlugin;

/// Plugins for the [`avatars`](self) module.
#[derive(Default)]
pub struct Avatars;

impl PluginGroup for Avatars {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>().add(HumanoidPlugin)
	}
}
