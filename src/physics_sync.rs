use avian3d::math::AsF32;
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use crate::networking::{RemotePlayer, UpdatePhysicsPosition};

pub struct PhysicsSyncNetworkingPlugin;

impl Plugin for PhysicsSyncNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_positions);
    }
}

fn sync_positions(mut event_reader: EventReader<UpdatePhysicsPosition>, mut players: Query<(&RemotePlayer, &mut Position, &mut Rotation, &mut LinearVelocity, &mut AngularVelocity)>) {
    for update in event_reader.read() {
        for (uuid, mut pos, mut rot, mut lin, mut ang) in players.iter_mut() {
            if uuid.0 != update.uuid {
                continue;
            }
            if pos.distance(*update.position) <= 0.01 {
                if rot.0.angle_between(update.rotation.0).abs() <= 0.01 {
                    continue;
                }
            }

            *pos = update.position.clone();
            *rot = update.rotation.clone();
            *lin = update.linear_velocity.clone();
            //*ang = update.angular_velocity.clone();
        }
    }

}