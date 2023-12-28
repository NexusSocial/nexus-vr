use bevy::{
	prelude::{
		debug, default, AssetServer, BuildChildren, Commands, Entity, Event,
		EventReader, Name, Plugin, Res, Update,
	},
	reflect::Reflect,
	scene::SceneBundle,
};
use bevy_vrm::VrmBundle;

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
	pub player: Entity,
	pub avi_url: String,
}

fn on_assign(
	mut cmds: Commands,
	mut select_evts: EventReader<AssignAvatar>,
	asset_server: Res<AssetServer>,
) {
	for evt @ AssignAvatar { player, avi_url } in select_evts.read() {
		debug!("{evt:?}");
		// let mut transform = Transform::from_xyz(0.0, -1.0, -4.0);
		// transform.rotate_y(PI);
		let bundle = VrmBundle {
			vrm: asset_server.load(avi_url),
			scene_bundle: SceneBundle { ..default() },
		};

		cmds.entity(*player).with_children(|parent| {
			parent.spawn((Name::from("Vrm"), bundle));
		});
	}
}
