//! Handles description of humanoid rigs.

use std::str::FromStr;

use color_eyre::{eyre::bail, Result};

use bevy::{
	log::{error, warn},
	prelude::{
		default, Assets, Children, Commands, Component, Entity, Event, EventReader,
		Handle, HierarchyQueryExt, Local, Name, Plugin, PreUpdate, Query, Res, Without,
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
	pub bone_map: BoneMap<Entity>,
}

#[derive(Reflect)]
pub struct BoneMap<T>(pub HashMap<BoneKind, T>);

impl<T> Default for BoneMap<T> {
	fn default() -> Self {
		Self(HashMap::default())
	}
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
	entity_names: Query<(Entity, &Name)>,
) {
	for &AutoAssignRigRequest { mesh: root_mesh } in evts.read() {
		let Ok(vrm_handle) = vrm_handles.get(root_mesh) else {
			// this system is only for vrm entities!
			continue;
		};
		let vrm = vrm_assets
			.get(vrm_handle)
			.expect("should be impossible for asset to not exist");

		let (vrm_kind, nodes_result) =
			match (&vrm.extensions.vrmc_vrm, &vrm.extensions.vrm0) {
				(Some(_), Some(_)) => {
					error!(
						"both vrm0 and vrm1 extension data was present for entity
                        {root_mesh:?}. Refusing to autoassign rig."
					);
					continue;
				}
				(None, None) => {
					error!(
						"both vrm0 and vrm1 extension data was missing for entity
                        {root_mesh:?}. Cannot autoassign rig."
					);
					continue;
				}
				(Some(vrm1), None) => ("vrm1", nodes_from_vrm1(vrm1)),
				(None, Some(vrm0)) => ("vrm0", nodes_from_vrm0(vrm0)),
			};

		let nodes = match nodes_result {
			Err(err) => {
				error!(
					"Failed to determine rig assignment from {vrm_kind} for entity
                    {root_mesh:?}: {err}"
				);
				continue;
			}
			Ok(rig) => rig,
		};

		let mut found_bones: HashMap<BoneKind, Entity> = default();
		for (b, idx) in nodes.0.into_iter() {
			let node_handle = &vrm.gltf.nodes[idx.0 as usize];
			let Some((node_name, _)) = vrm
				.gltf
				.named_nodes
				.iter()
				.find(|(_str, handle)| *handle == node_handle)
			else {
				panic!("no gltf node exists at the index {}", idx.0);
			};

			let Some(entity) = entity_names.iter().find_map(|(entity, entity_name)| {
				(entity_name.as_str() == node_name).then_some(entity)
			}) else {
				panic!("no entity with the same name as the node ({node_name}) exists");
			};

			found_bones.insert(b, entity);
		}

		cmds.entity(root_mesh).insert(HumanoidRig {
			bone_map: BoneMap(found_bones),
		});
	}
}

#[derive(Clone, Copy)]
struct GltfNodeIdx(u32);

fn nodes_from_vrm0(
	vrm: &bevy_vrm::extensions::vrm0::Vrm,
) -> Result<BoneMap<GltfNodeIdx>> {
	let Some(hb) = &vrm.humanoid.human_bones else {
		bail!("vrm0 human_bones data missing")
	};

	let mut nodes: BoneMap<GltfNodeIdx> = default();
	for b in hb {
		let Some(name) = b.name.as_deref() else {
			continue;
		};
		let Some(node) = b.node.map(GltfNodeIdx) else {
			continue;
		};
		let Ok(kind) = name.parse::<BoneKind>() else {
			continue;
		};
		nodes.0.insert(kind, node);
	}
	Ok(nodes)
}

fn nodes_from_vrm1(
	_vrm: &bevy_vrm::extensions::vrmc_vrm::VrmcVrm,
) -> Result<BoneMap<GltfNodeIdx>> {
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
		warn!(
			"Guessing humanoid rig mapping for {root_mesh:?}, this is likely to 
            be inaccurate though"
		);

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
		// TODO: Do something smarter than just picking the last map entry.
		let winners = BoneMap(
			map.into_iter()
				.map(|(entity, bone)| (bone, entity))
				.collect(),
		);

		cmds.entity(root_mesh)
			.insert(HumanoidRig { bone_map: winners });
	}
}

macro_rules! enum_helper {
    (
        $(#[$($attr:meta),*])*
        $vis:vis enum $ident:ident {
            $($variant:ident,)*
        }
    ) => {
        $(#[$($attr),*])*
        $vis enum $ident {
            $($variant,)*
        }

        impl FromStr for $ident {
            type Err = ();

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(match s {
                    $(s if s.eq_ignore_ascii_case(stringify!($variant)) => Self::$variant,)*
                    _ => return Err(()),
                })
            }
        }
    }
}

enum_helper! {
	/// See also https://github.com/vrm-c/vrm-specification/blob/57ba30bcb2638617d5ee655e35444caa299971bd/specification/VRMC_vrm-1.0/humanoid.md
	#[derive(Reflect, Debug, Eq, PartialEq, Copy, Clone, Hash)]
	pub enum BoneKind {
		Neck,
		Hips,
		Head,
		Spine,
		Chest,
		RightUpperArm,
		LeftUpperArm,
		RightLowerArm,
		LeftLowerArm,
		RightHand,
		LeftHand,
		RightUpperLeg,
		LeftUpperLeg,
		RightFoot,
		LeftFoot,
	}
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
