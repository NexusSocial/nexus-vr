use crate::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use std::time::Duration;
use tracing::Level;

pub const CLIENT_PORT: u16 = 6000;
pub const SERVER_PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

pub const KEY: Key = [0; 32];

pub fn shared_config() -> SharedConfig {
	SharedConfig {
        enable_replication: true,
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(100),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        log: LogConfig {
            level: Level::INFO,
            filter: "wgpu=error,wgpu_hal=error,naga=warn,bevy_app=info,bevy_render=warn,quinn=warn"
                .to_string(),
        },
    }
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
	fn build(&self, app: &mut App) {
		// app.add_plugins(WorldInspectorPlugin::new());
		app.add_systems(Update, draw_boxes);
	}
}

// This system defines how we update the player's positions when we receive an input
pub fn shared_movement_behaviour(position: &mut PlayerPosition, input: &Inputs) {
	const MOVE_SPEED: f32 = 0.01;
	#[allow(clippy::single_match)]
	match input {
		Inputs::Direction(direction) => {
			if direction.up {
				position.y += MOVE_SPEED;
			}
			if direction.down {
				position.z -= MOVE_SPEED;
			}
			if direction.left {
				position.x -= MOVE_SPEED;
			}
			if direction.right {
				position.z += MOVE_SPEED;
			}
		}
		_ => {}
	}
}

/// System that draws the boxed of the player positions.
/// The components should be replicated from the server to the client
pub fn draw_boxes(mut gizmos: Gizmos, players: Query<(&PlayerPosition, &PlayerColor)>) {
	for (position, color) in &players {
		gizmos.rect(
			Vec3::new(position.x, position.y, position.z),
			Quat::IDENTITY,
			Vec2::ONE * 0.1,
			color.0,
		);
	}
}
