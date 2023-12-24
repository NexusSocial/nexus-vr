use crate::macro_utils::unwrap_or_continue;

use bevy::{
	prelude::{
		Children, Commands, Component, Entity, Event, EventReader, HierarchyQueryExt,
		Local, Name, Plugin, Query, Update,
	},
	utils::HashMap,
};

#[derive(Default)]
pub struct HumanoidPlugin;

impl Plugin for HumanoidPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_event::<AutoAssignRigRequest>()
			.add_systems(Update, auto_rig_assignment);
	}
}

#[derive(Component)]
pub struct HumanoidRig {
	pub entities: Data<Entity>,
}

#[derive(Default)]
pub struct Data<T> {
	pub head: T,
	pub hand_l: T,
	pub hand_r: T,
	// TODO: Specify rest of skeleton
}

/// An event that when fired, runs [`auto_rig_assignment`].
#[derive(Event, Debug)]
pub struct AutoAssignRigRequest {
	pub mesh: Entity,
}

/// Attempts to automatically assign the rig to the mesh in [`AutoAssignRigEvent`].
pub fn auto_rig_assignment(
	mut cmds: Commands,
	mut evts: EventReader<AutoAssignRigRequest>,
	children: Query<&Children>,
	names: Query<&Name>,
	mut str_buf: Local<String>,
) {
	for evt in evts.read() {
		let mut map = HashMap::new();
		let root_mesh = evt.mesh;
		// TODO: Switch to a dfs and keep track of the side of the body as a hint
		for c in children.iter_descendants(root_mesh) {
			if let Ok(name) = names.get(c) {
				// TODO: Use the side of body as a hint instead of always passing
				// `None`
				if let Ok(kind) = guess_bone_kind(name, &mut str_buf, None) {
					map.insert(c, kind);
				}
			}
		}

		// Now that we have a map of entity -> BoneKind as a list of candidates,
		// pick winners and reverse the map to be BoneKind -> Entity.
		// TODO: Do something smarter than just picking the first map entry.
		let winners = Data {
			head: unwrap_or_continue!(map
				.iter()
				.find_map(|(k, v)| (*v == BoneKind::Head).then_some(*k))),
			hand_l: unwrap_or_continue!(map
				.iter()
				.find_map(|(k, v)| (*v == BoneKind::LeftHand).then_some(*k))),
			hand_r: unwrap_or_continue!(map
				.iter()
				.find_map(|(k, v)| (*v == BoneKind::RightHand).then_some(*k))),
		};

		cmds.entity(root_mesh)
			.insert(HumanoidRig { entities: winners });
	}
}

// See also https://github.com/vrm-c/vrm-specification/blob/57ba30bcb2638617d5ee655e35444caa299971bd/specification/VRMC_vrm-1.0/humanoid.md
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum BoneKind {
	Hips,
	Head,
	RightHand,
	LeftHand,
	RightFoot,
	LeftFoot,
}

/// `scratch` is a working scratch space used to avoid allocations
/// `side_hint` is a hint about which side of the body it is.
fn guess_bone_kind(
	s: &str,
	scratch: &mut String,
	side_hint: Option<Side>,
) -> Result<BoneKind, ()> {
	scratch.clear();
	scratch.push_str(s);
	scratch.make_ascii_lowercase();
	Ok(match scratch {
		s if s.contains("head") => BoneKind::Head,
		s if s.contains("hip") => BoneKind::Hips,
		s if s.contains("hand") => match side_hint {
			Some(Side::Left) => BoneKind::LeftHand,
			Some(Side::Right) => BoneKind::RightHand,

			None => match detect_side(s, "hand") {
				Some(Side::Left) => BoneKind::LeftHand,
				Some(Side::Right) => BoneKind::RightHand,
				None => return Err(()),
			},
		},
		s if s.contains("foot") => match side_hint {
			Some(Side::Left) => BoneKind::LeftFoot,
			Some(Side::Right) => BoneKind::RightFoot,

			None => match detect_side(s, "foot") {
				Some(Side::Left) => BoneKind::LeftFoot,
				Some(Side::Right) => BoneKind::RightFoot,
				None => return Err(()),
			},
		},
		_ => return Err(()),
	})
}

/// A side of the body.
enum Side {
	Left,
	Right,
}

/// Attempts to find which side of the body this bodypart is.
/// `s` is the string to identify.
/// `body_part` is the substring which is the name of the bodypart.
///
/// May return `None` if we could not detect the side.
fn detect_side(s: &str, body_part: &str) -> Option<Side> {
	let (prefix, suffix) = s
		.split_once(body_part)
		.expect("`body_part` argument should be present as a substring");

	// TODO: This logic can be improved but ill keep it for now
	if prefix.contains('l') || suffix.contains('l') {
		Some(Side::Left)
	} else if prefix.contains('r') || suffix.contains('r') {
		Some(Side::Right)
	} else {
		None
	}
}
