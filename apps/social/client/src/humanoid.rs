use bevy::prelude::{Component, Entity, Event, EventReader, Plugin, Update};

#[derive(Default)]
pub struct HumanoidPlugin;

impl Plugin for HumanoidPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_event::<AutoAssignRigEvent>()
			.add_systems(Update, auto_rig_assignment);
		//
	}
}

#[derive(Component)]
pub struct Viewpoint {}

#[derive(Component)]
pub struct HumanoidRig {
	pub entities: Data<Entity>,
}

pub struct Data<T> {
	pub head: T,
	pub hand_l: T,
	pub hand_r: T,
	// TODO: Specify rest of skeleton
}

/// When fired, runs [`auto_rig_assignment`]
#[derive(Event)]
pub struct AutoAssignRigEvent {
	pub mesh: Entity,
}

/// Attempts to automatically assign the rig to the mesh in [`AutoAssignRigEvent`].
pub fn auto_rig_assignment(mut evts: EventReader<AutoAssignRigEvent>) {
	for _evt in evts.read() {
		// TODO
	}
}
