//! The server's understanding of information about individual players.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

/// Plugins for the [`player_management`](self) module.
#[derive(Default)]
pub struct PlayerManagement;

impl PluginGroup for PlayerManagement {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
	}
}