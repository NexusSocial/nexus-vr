//! Handles description of humanoid rigs.

use crate::macro_utils::unwrap_or_continue;
use color_eyre::{eyre::bail, Result};

use bevy::{
	log::{error, warn},
	prelude::{
		Assets, Children, Commands, Component, Entity, Event, EventReader, Handle,
		HierarchyQueryExt, Local, Name, Plugin, PreUpdate, Query, Res, Without,
	},
	reflect::Reflect,
	utils::HashMap,
};
use bevy_vrm::Vrm;

#[derive(Default)]
pub struct HumanoidPlugin;

impl Plugin for HumanoidPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<HumanoidRig>()
			.add_event::<AutoAssignRigRequest>()
			// TODO: PreUpdate seemed to fix nondeterminism, check if its necessary
			.add_systems(PreUpdate, autoassign_from_names)
			.add_systems(PreUpdate, autoassign_from_vrm);
	}
}

#[derive(Reflect, Component)]
pub struct HumanoidRig {
	pub entities: Data<Entity>,
}

#[derive(Reflect, Default)]
pub struct Data<T> {
	pub head: T,
	pub hand_l: T,
	pub hand_r: T,
	// TODO: Specify rest of skeleton
}

/// An event that when fired, automatically inserts a [`HumanoidRig`] component with
/// autodetected mappings.
#[derive(Event, Debug)]
pub struct AutoAssignRigRequest {
	pub mesh: Entity,
}

fn autoassign_from_vrm(
	mut cmds: Commands,
	mut evts: EventReader<AutoAssignRigRequest>,
	vrm_handles: Query<&Handle<Vrm>>,
	vrm_assets: Res<Assets<Vrm>>,
) {
	for &AutoAssignRigRequest { mesh: root_mesh } in evts.read() {
		let Ok(vrm_handle) = vrm_handles.get(root_mesh) else {
			// this system is only for vrm entities!
			continue;
		};
		let vrm = vrm_assets
			.get(vrm_handle)
			.expect("should be impossible for asset to not exist");

		let (vrm_kind, rig_result) = match (
			&vrm.extensions.vrmc_vrm,
			&vrm.extensions.vrm0,
		) {
			(Some(_), Some(_)) => {
				error!("both vrm0 and vrm1 extension data was present for entity {root_mesh:?}. Refusing to autoassign rig.");
				continue;
			}
			(None, None) => {
				error!("both vrm0 and vrm1 extension data was missing for entity {root_mesh:?}. Cannot autoassign rig.");
				continue;
			}
			(Some(vrm1), None) => ("vrm1", rig_from_vrm1(vrm1)),
			(None, Some(vrm0)) => ("vrm0", rig_from_vrm0(vrm0)),
		};

		match rig_result {
			Err(err) => {
				error!("Failed to determine rig assignment from {vrm_kind} for entity {root_mesh:?}: {err}");
				continue;
			}
			Ok(rig) => {
				cmds.entity(root_mesh).insert(rig);
			}
		}
	}
}

fn rig_from_vrm0(vrm: &bevy_vrm::extensions::vrm0::Vrm) -> Result<HumanoidRig> {
	let Some(_hb) = &vrm.humanoid.human_bones else {
		bail!("vrm0 human_bones data missing")
	};
	bail!("todo")
}

fn rig_from_vrm1(
	_vrm: &bevy_vrm::extensions::vrmc_vrm::VrmcVrm,
) -> Result<HumanoidRig> {
	bail!("todo")
}

/// Automatically assigns the rig based on entity names.
fn autoassign_from_names(
	mut cmds: Commands,
	mut evts: EventReader<AutoAssignRigRequest>,
	children: Query<&Children>,
	names: Query<&Name>,
	mut str_buf: Local<String>,
	non_vrm: Query<Without<Handle<Vrm>>>,
) {
	for &AutoAssignRigRequest { mesh: root_mesh } in evts.read() {
		// this system is only needed for non vrm stuff
		if !non_vrm.contains(root_mesh) {
			continue;
		}

		warn!("Guessing humanoid rig mapping for {root_mesh:?}, this is likely to be inaccurate though");

		let mut map = HashMap::new();
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
