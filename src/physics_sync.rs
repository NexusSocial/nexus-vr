use crate::networking::{RemotePlayer, UpdatePhysicsPosition};
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;

pub struct PhysicsSyncNetworkingPlugin;

impl Plugin for PhysicsSyncNetworkingPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, sync_positions);
	}
}

fn sync_positions(
	mut event_reader: EventReader<UpdatePhysicsPosition>,
	mut players: Query<(
		&RemotePlayer,
		&mut Position,
		&mut Rotation,
		&mut LinearVelocity,
		&mut AngularVelocity,
	)>,
) {
	for update in event_reader.read() {
		for (uuid, mut pos, mut rot, mut lin, _) in players.iter_mut() {
			if uuid.0 != update.uuid {
				continue;
			}
			// This position check is one centimeter?! that's a little big no?
			if pos.distance(*update.position) <= 0.01
				&& rot.0.angle_between(update.rotation.0).abs() <= 0.01
			{
				continue;
			}

			*pos = update.position;
			*rot = update.rotation;
			*lin = update.linear_velocity;
			//*ang = update.angular_velocity.clone();
		}
	}
}
