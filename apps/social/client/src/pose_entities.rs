use crate::avatars::DmEntity;
use bevy::prelude::*;
use derive_more::Display;
use social_networking::data_model::{Isometry, PlayerPose};

pub struct PoseEntitiesPlugin;
impl Plugin for PoseEntitiesPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreUpdate, sync_with_avatar);
		app.add_systems(PreUpdate, setup_pose_roots);
	}
}

fn sync_with_avatar(
	pose_query: Query<&PlayerPose>,
	local_avi_query: Query<(Entity, &DmEntity), With<Root>>,
	avatar_children: Query<&Children>,
	mut left_hand_transform: Query<
		(&mut Transform, &mut GlobalPose),
		(With<LeftHand>, Without<RightHand>, Without<Head>),
	>,
	mut right_hand_transform: Query<
		(&mut Transform, &mut GlobalPose),
		(Without<LeftHand>, With<RightHand>, Without<Head>),
	>,
	mut head_transform: Query<
		(&mut Transform, &mut GlobalPose),
		(Without<LeftHand>, Without<RightHand>, With<Head>),
	>,
) {
	for (avi_entity, avi_dm) in &local_avi_query {
		let pose = match pose_query.get(avi_dm.0) {
			Ok(v) => v,
			Err(_) => continue,
		};
		for e in avatar_children.iter_descendants(avi_entity) {
			if let Ok((mut transform, mut global_pose)) = left_hand_transform.get_mut(e)
			{
				transform.translation = pose.hand_l.trans;
				transform.rotation = pose.hand_l.rot;
				**global_pose = pose
					.root
					.mul_isometry(pose.hand_l.scale_translation(pose.root_scale));
			}
			if let Ok((mut transform, mut global_pose)) =
				right_hand_transform.get_mut(e)
			{
				transform.translation = pose.hand_r.trans;
				transform.rotation = pose.hand_r.rot;
				**global_pose = pose
					.root
					.mul_isometry(pose.hand_r.scale_translation(pose.root_scale));
			}
			if let Ok((mut transform, mut global_pose)) = head_transform.get_mut(e) {
				transform.translation = pose.head.trans;
				transform.rotation = pose.head.rot;
				**global_pose = pose
					.root
					.mul_isometry(pose.head.scale_translation(pose.root_scale));
			}
		}
	}
}

fn setup_pose_roots(mut cmds: Commands, roots: Query<Entity, With<SetupRoot>>) {
	for e in &roots {
		cmds.entity(e).insert(Root);
		let head = cmds.spawn(Head).id();
		let left_hand = cmds.spawn(LeftHand).id();
		let right_hand = cmds.spawn(RightHand).id();
		cmds.entity(e).push_children(&[head, left_hand, right_hand]);
	}
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut)]
pub struct GlobalPose(Isometry);

#[derive(Component, Clone, Copy, Debug, Default, Display)]
pub struct SetupRoot;

#[derive(Component, Clone, Copy, Debug, Default, Display)]
pub struct LeftHand;
#[derive(Component, Clone, Copy, Debug, Default, Display)]
pub struct RightHand;
#[derive(Component, Clone, Copy, Debug, Default, Display)]
pub struct Head;
#[derive(Component, Clone, Copy, Debug, Default, Display)]
pub struct Root;
