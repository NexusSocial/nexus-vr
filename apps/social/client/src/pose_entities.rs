use crate::{
	avatars::DmEntity,
	custom_audio::{
		audio_output::AudioOutput,
		spatial_audio::{self, SpatialAudioListener},
	},
};
use bevy::prelude::*;
use derive_more::Display;
use social_networking::data_model::PlayerPose;

pub struct PoseEntitiesPlugin;
impl Plugin for PoseEntitiesPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreUpdate, sync_with_avatar);
		app.add_systems(PreUpdate, setup_pose_roots);
		app.add_systems(PreUpdate, move_audio_sinks);
	}
}

fn move_audio_sinks(
	mut cmds: Commands,
	local_avi_query: Query<
		(Entity, Has<SpatialAudioListener>),
		(With<Root>, With<spatial_audio::SpatialAudioSink>),
	>,
	audio_output: Res<AudioOutput>,
	avatar_children: Query<&Children>,
	head_query: Query<Entity, With<Head>>,
) {
	for (root, listener) in &local_avi_query {
		for child in avatar_children.iter_descendants(root) {
			if let Ok(head) = head_query.get(child) {
				cmds.entity(root)
					.remove::<spatial_audio::SpatialAudioSink>();
				cmds.entity(head).insert(spatial_audio::SpatialAudioSink {
					sink: rodio::SpatialSink::try_new(
						audio_output.stream_handle.as_ref().unwrap(),
						[0.0, 0.0, 0.0],
						(Vec3::X * 4.0 / -2.0).to_array(),
						(Vec3::X * 4.0 / 2.0).to_array(),
					)
					.unwrap(),
				});
				if listener {
					cmds.entity(root).remove::<SpatialAudioListener>();
					cmds.entity(head).insert(SpatialAudioListener);
				}
			}
		}
	}
}

fn sync_with_avatar(
	pose_query: Query<&PlayerPose>,
	local_avi_query: Query<(Entity, &DmEntity), With<Root>>,
	avatar_children: Query<&Children>,
	mut left_hand_transform: Query<
		&mut Transform,
		(With<LeftHand>, Without<RightHand>, Without<Head>),
	>,
	mut right_hand_transform: Query<
		&mut Transform,
		(Without<LeftHand>, With<RightHand>, Without<Head>),
	>,
	mut head_transform: Query<
		&mut Transform,
		(Without<LeftHand>, Without<RightHand>, With<Head>),
	>,
) {
	for (avi_entity, avi_dm) in &local_avi_query {
		let pose = match pose_query.get(avi_dm.0) {
			Ok(v) => v,
			Err(_) => continue,
		};
		for e in avatar_children.iter_descendants(avi_entity) {
			if let Ok(mut transform) = left_hand_transform.get_mut(e) {
				transform.translation = pose.hand_l.trans;
				transform.rotation = pose.hand_l.rot;
			}
			if let Ok(mut transform) = right_hand_transform.get_mut(e) {
				transform.translation = pose.hand_r.trans;
				transform.rotation = pose.hand_r.rot;
			}
			if let Ok(mut transform) = head_transform.get_mut(e) {
				transform.translation = pose.head.trans;
				transform.rotation = pose.head.rot;
			}
		}
	}
}

fn setup_pose_roots(mut cmds: Commands, roots: Query<Entity, With<SetupRoot>>) {
	for e in &roots {
		cmds.entity(e).insert(Root);
		cmds.entity(e).remove::<SetupRoot>();
		let head = cmds
			.spawn(Head)
			.insert(SpatialBundle::default())
			.insert(Name::new("Head"))
			.id();
		let left_hand = cmds
			.spawn(LeftHand)
			.insert(SpatialBundle::default())
			.insert(Name::new("Left Hand"))
			.id();
		let right_hand = cmds
			.spawn(RightHand)
			.insert(SpatialBundle::default())
			.insert(Name::new("Right Hand"))
			.id();
		cmds.entity(e).push_children(&[head, left_hand, right_hand]);
	}
}

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
