use crate::avatars::loading::FullyLoadedAvatar;
use bevy::prelude::{info, Children, Handle, HierarchyQueryExt, Query};
use bevy::{
	prelude::{
		debug, default, AssetServer, BuildChildren, Commands, Entity, Event,
		EventReader, Name, Plugin, Res, Update,
	},
	reflect::Reflect,
	scene::SceneBundle,
};
use bevy_vrm::{Vrm, VrmBundle};
use social_common::humanoid::HumanoidRig;

pub struct AvatarSelectPlugin;

impl Plugin for AvatarSelectPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<AssignAvatar>()
			.add_event::<AssignAvatar>()
			.add_systems(Update, on_assign);
	}
}

#[derive(Debug, Reflect, Event)]
pub struct AssignAvatar {
	/// The player
	pub avi_entity: Entity,
	pub avi_url: String,
}

fn on_assign(
	mut cmds: Commands,
	mut select_evts: EventReader<AssignAvatar>,
	children: Query<&Children>,
	vrms: Query<&Handle<Vrm>>,
	asset_server: Res<AssetServer>,
) {
	for evt @ AssignAvatar {
		avi_entity: player,
		avi_url,
	} in select_evts.read()
	{
		info!("assign avatar event: {evt:?}");
		// let mut transform = Transform::from_xyz(0.0, -1.0, -4.0);
		// transform.rotate_y(PI);

		let mut already_has_vrm = false;

		for child in children.iter_descendants(*player) {
			if vrms.contains(child) {
				let bundle = VrmBundle {
					vrm: asset_server.load(avi_url),
					scene_bundle: SceneBundle { ..default() },
				};
				cmds.entity(child).insert(bundle);
				cmds.entity(child).remove::<HumanoidRig>();
				cmds.entity(child).remove::<FullyLoadedAvatar>();
				cmds.entity(child)
					.remove::<crate::avatars::loading::RigRequestSent>();
				already_has_vrm = true;
				info!("already has vrm, so replacing bundle, removing: [HumanoidRig, FullyLoadedAvatar, RigRequestSent]");
				break;
			}
		}
		if already_has_vrm {
			continue;
		}
		let bundle = VrmBundle {
			vrm: asset_server.load(avi_url),
			scene_bundle: SceneBundle { ..default() },
		};
		info!("New VRM, spawning child");
		cmds.entity(*player).with_children(|parent| {
			parent.spawn((Name::from("Vrm"), bundle));
		});
	}
}
