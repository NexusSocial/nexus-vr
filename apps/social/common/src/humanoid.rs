//! Handles description of humanoid rigs.

use std::str::FromStr;

use bevy::transform::components::{GlobalTransform, Transform};
use color_eyre::{eyre, eyre::bail, Result};
use eyre::eyre;

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
use tracing::info;

#[derive(Default)]
pub struct HumanoidPlugin;

impl Plugin for HumanoidPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.register_type::<HumanoidRig>()
			.add_event::<AutoAssignRigRequest>()
			// TODO: PreUpdate seemed to fix nondeterminism, check if its necessary
			.add_systems(PreUpdate, autoassign);
	}
}

#[derive(Reflect, Component)]
pub struct HumanoidRig {
	pub bone_map: BoneMap<Entity>,
	pub entities: [Option<Entity>; SKELETON_ARR_LEN],
	pub height: f32,
	pub defaults: Skeleton,
	pub root_defaults: Skeleton,
}

#[derive(Reflect)]
pub struct BoneMap<T>(pub HashMap<BoneKind, T>);

impl<T> Default for BoneMap<T> {
	fn default() -> Self {
		Self(HashMap::default())
	}
}

const HEAD: usize = 0;
const HIPS: usize = 1;
const SPINE: usize = 2;
const CHEST: usize = 3;
const UPPER_CHEST: usize = 4;
const NECK: usize = 5;
const LEFT_SHOULDER: usize = 6;
const LEFT_EYE: usize = 7;
const LEFT_FOOT: usize = 8;
const LEFT_HAND: usize = 9;
const LEFT_LEG_UPPER: usize = 10;
const LEFT_LEG_LOWER: usize = 11;
const LEFT_ARM_UPPER: usize = 12;
const LEFT_ARM_LOWER: usize = 13;
const RIGHT_SHOULDER: usize = 14;
const RIGHT_EYE: usize = 15;
const RIGHT_FOOT: usize = 16;
const RIGHT_HAND: usize = 17;
const RIGHT_LEG_UPPER: usize = 18;
const RIGHT_LEG_LOWER: usize = 19;
const RIGHT_ARM_UPPER: usize = 20;
const RIGHT_ARM_LOWER: usize = 21;
pub const SKELETON_ARR_LEN: usize = 22;

const SKELETON_ARR_BONE_KIND: [BoneKind; SKELETON_ARR_LEN] = [
	BoneKind::Head,
	BoneKind::Hips,
	BoneKind::Spine,
	BoneKind::Chest,
	BoneKind::UpperChest,
	BoneKind::Neck,
	BoneKind::LeftShoulder,
	BoneKind::LeftEye,
	BoneKind::LeftFoot,
	BoneKind::LeftHand,
	BoneKind::LeftUpperLeg,
	BoneKind::LeftLowerLeg,
	BoneKind::LeftUpperArm,
	BoneKind::LeftLowerArm,
	BoneKind::RightShoulder,
	BoneKind::RightEye,
	BoneKind::RightFoot,
	BoneKind::RightHand,
	BoneKind::RightUpperLeg,
	BoneKind::RightLowerLeg,
	BoneKind::RightUpperArm,
	BoneKind::RightLowerArm,
];

#[derive(Reflect, Copy, Clone)]
pub struct Skeleton {
	pub head: Transform,
	pub hips: Transform,
	pub spine: Transform,
	pub chest: Option<Transform>,
	pub upper_chest: Option<Transform>,
	pub neck: Option<Transform>,
	pub left: SkeletonSide,
	pub right: SkeletonSide,
}

#[derive(Reflect, Copy, Clone)]
pub struct SkeletonSide {
	pub shoulder: Option<Transform>,
	pub eye: Transform,
	pub leg: Limb,
	pub arm: Limb,
	pub foot: Transform,
	pub hand: Transform,
}

#[derive(Reflect, Copy, Clone)]
pub struct Limb {
	pub upper: Transform,
	pub lower: Transform,
}

impl Skeleton {
	pub fn new(
		skeleton: [Option<Transform>; SKELETON_ARR_LEN],
	) -> color_eyre::Result<Self> {
		Ok(Self {
			head: skeleton[HEAD].ok_or(eyre!("bad Skeleton, missing head"))?,
			hips: skeleton[HIPS].ok_or(eyre!("bad Skeleton, missing hips"))?,
			spine: skeleton[SPINE].ok_or(eyre!("bad Skeleton, missing spine"))?,
			chest: skeleton[CHEST],
			upper_chest: skeleton[UPPER_CHEST],
			neck: skeleton[NECK],
			left: SkeletonSide {
				shoulder: skeleton[LEFT_SHOULDER],
				eye: skeleton[LEFT_EYE]
					.ok_or(eyre!("bad Skeleton, missing left eye"))?,
				foot: skeleton[LEFT_FOOT]
					.ok_or(eyre!("bad Skeleton, missing left foot"))?,
				hand: skeleton[LEFT_HAND]
					.ok_or(eyre!("bad Skeleton, missing left hand"))?,
				leg: Limb {
					upper: skeleton[LEFT_LEG_UPPER]
						.ok_or(eyre!("bad Skeleton, missing upper left leg"))?,
					lower: skeleton[LEFT_LEG_LOWER]
						.ok_or(eyre!("bad Skeleton, missing lower left leg"))?,
				},
				arm: Limb {
					upper: skeleton[LEFT_ARM_UPPER]
						.ok_or(eyre!("bad Skeleton, missing upper left arm"))?,
					lower: skeleton[LEFT_ARM_LOWER]
						.ok_or(eyre!("bad Skeleton, missing lower left leg"))?,
				},
			},
			right: SkeletonSide {
				shoulder: skeleton[RIGHT_SHOULDER],
				eye: skeleton[RIGHT_EYE]
					.ok_or(eyre!("bad Skeleton, missing right eye"))?,
				foot: skeleton[RIGHT_FOOT]
					.ok_or(eyre!("bad Skeleton, missing right foot"))?,
				hand: skeleton[RIGHT_HAND]
					.ok_or(eyre!("bad Skeleton, missing right hand"))?,
				leg: Limb {
					upper: skeleton[RIGHT_LEG_UPPER]
						.ok_or(eyre!("bad Skeleton, missing upper right leg"))?,
					lower: skeleton[RIGHT_LEG_LOWER]
						.ok_or(eyre!("bad Skeleton, missing lower right leg"))?,
				},
				arm: Limb {
					upper: skeleton[RIGHT_ARM_UPPER]
						.ok_or(eyre!("bad Skeleton, missing upper right arm"))?,
					lower: skeleton[RIGHT_ARM_LOWER]
						.ok_or(eyre!("bad Skeleton, missing lower right arm"))?,
				},
			},
		})
	}
	pub fn arrayify(self) -> [Option<Transform>; SKELETON_ARR_LEN] {
		let mut skeleton: [Option<Transform>; SKELETON_ARR_LEN] = Default::default();
		skeleton[HEAD] = Some(self.head);
		skeleton[HIPS] = Some(self.hips);
		skeleton[SPINE] = Some(self.spine);
		skeleton[CHEST] = self.chest;
		skeleton[UPPER_CHEST] = self.upper_chest;
		skeleton[NECK] = self.neck;
		skeleton[LEFT_SHOULDER] = self.left.shoulder;
		skeleton[LEFT_EYE] = Some(self.left.eye);
		skeleton[LEFT_FOOT] = Some(self.left.foot);
		skeleton[LEFT_HAND] = Some(self.left.hand);
		skeleton[LEFT_ARM_UPPER] = Some(self.left.arm.upper);
		skeleton[LEFT_ARM_LOWER] = Some(self.left.arm.lower);
		skeleton[LEFT_LEG_UPPER] = Some(self.left.leg.upper);
		skeleton[LEFT_LEG_LOWER] = Some(self.left.leg.lower);
		skeleton[RIGHT_SHOULDER] = self.right.shoulder;
		skeleton[RIGHT_EYE] = Some(self.right.eye);
		skeleton[RIGHT_FOOT] = Some(self.right.foot);
		skeleton[RIGHT_HAND] = Some(self.right.hand);
		skeleton[RIGHT_ARM_UPPER] = Some(self.right.arm.upper);
		skeleton[RIGHT_ARM_LOWER] = Some(self.right.arm.lower);
		skeleton[RIGHT_LEG_UPPER] = Some(self.right.leg.upper);
		skeleton[RIGHT_LEG_LOWER] = Some(self.right.leg.lower);
		skeleton
	}
	pub fn recalculate_root(mut self) -> Self {
		self.spine = self.hips.mul_transform(self.spine);
		let mut highest_root = self.spine;
		if let Some(t) = self.chest {
			let result = self.spine.mul_transform(t);
			self.chest = Some(result);
			highest_root = result;
			if let Some(t2) = self.upper_chest {
				let result = highest_root.mul_transform(t2);
				self.upper_chest = Some(result);
				highest_root = result;
			}
		}
		let mut head_root = highest_root;
		if let Some(t) = self.neck {
			let result = highest_root.mul_transform(t);
			self.neck = Some(result);
			head_root = result;
		}
		self.head = head_root.mul_transform(self.head);
		self.left = self
			.left
			.recalculate_root(self.head, self.hips, highest_root);
		self.right = self
			.right
			.recalculate_root(self.head, self.hips, highest_root);
		self
	}
}

impl SkeletonSide {
	pub fn recalculate_root(
		mut self,
		head: Transform,
		hips: Transform,
		mut highest_root: Transform,
	) -> Self {
		self.eye = head.mul_transform(self.eye);
		if let Some(t) = self.shoulder {
			let result = highest_root.mul_transform(t);
			self.shoulder = Some(result);
			highest_root = result;
		}
		self.arm = self.arm.recalculate_root(highest_root);
		self.leg = self.leg.recalculate_root(hips);
		self.hand = self.arm.lower.mul_transform(self.hand);
		self.foot = self.leg.lower.mul_transform(self.foot);
		self
	}
}

impl Limb {
	pub fn recalculate_root(mut self, root: Transform) -> Self {
		self.upper = root.mul_transform(self.upper);
		self.lower = self.upper.mul_transform(self.lower);
		self
	}
}

/// An event that when fired, automatically inserts a [`HumanoidRig`] component with
/// autodetected mappings.
#[derive(Event, Debug)]
pub struct AutoAssignRigRequest {
	pub mesh: Entity,
}

fn autoassign(
	mut cmds: Commands,
	mut evts: EventReader<AutoAssignRigRequest>,
	vrm_handles: Query<&Handle<Vrm>>,
	non_vrm: Query<Without<Handle<Vrm>>>,
	vrm_assets: Res<Assets<Vrm>>,
	entity_names: Query<(Entity, &Name)>,
	children: Query<&Children>,
	mut str_buf: Local<String>,
	transforms: Query<(&Transform, &GlobalTransform)>,
) {
	for &AutoAssignRigRequest { mesh: root_mesh } in evts.read() {
		info!("auto assign rig request received for: {:?}", root_mesh);
		let found_bones_result: Result<HashMap<BoneKind, Entity>> = match vrm_handles
			.get(root_mesh)
		{
			Ok(vrm_handle) => {
				let vrm = vrm_assets
					.get(vrm_handle)
					.expect("should be impossible for asset to not exist");
				autoassign_from_vrm(root_mesh, vrm, &children, &entity_names)
			}
			Err(_) => {
				if !non_vrm.contains(root_mesh) {
					continue;
				}
				autoassign_from_names(root_mesh, &children, &entity_names, &mut str_buf)
			}
		};
		let found_bones = match found_bones_result {
			Ok(m) => m,
			Err(e) => {
				error!("{e}");
				continue;
			}
		};
		for pair in found_bones.clone().into_iter() {
			cmds.entity(pair.1).insert(pair.0);
		}
		let mut skel_entities: [Option<Entity>; SKELETON_ARR_LEN] =
			[None; SKELETON_ARR_LEN];
		let mut skeleton_transform_array: [Option<Transform>; SKELETON_ARR_LEN] =
			[None; SKELETON_ARR_LEN];
		let mut skeleton_root_array: [Option<Transform>; SKELETON_ARR_LEN] =
			[None; SKELETON_ARR_LEN];
		let root_t = match transforms.get(root_mesh) {
			Ok(t) => t.1,
			Err(e) => {
				error!("Avatar root has no associated transform, {e}");
				continue;
			}
		};
		for i in 0..SKELETON_ARR_LEN {
			let entity = match found_bones.get(&SKELETON_ARR_BONE_KIND[i]) {
				Some(e) => *e,
				None => continue,
			};
			skel_entities[i] = Some(entity);
			skeleton_transform_array[i] = Some(match transforms.get(entity) {
				Ok(t) => *t.0,
				Err(_) => continue,
			});
			skeleton_root_array[i] = Some(match transforms.get(entity) {
				Ok(t) => t.1.reparented_to(root_t),
				Err(_) => continue,
			});
		}
		let skel = match Skeleton::new(skeleton_transform_array) {
			Ok(skel) => skel,
			Err(err) => {
				error!("Transformation to skeleton failed, {err}");
				continue;
			}
		};
		// this next thing should really never fail, that would be very odd.
		let root_skel = match Skeleton::new(skeleton_root_array) {
			Ok(skel) => skel,
			Err(err) => {
				error!("GlobalTransform Transform mismatch, {err}");
				continue;
			}
		};
		let rig = HumanoidRig {
			bone_map: BoneMap(found_bones),
			entities: skel_entities,
			height: ((root_skel.left.eye.translation
				+ root_skel.right.eye.translation)
				/ 2.)
				.length(),
			defaults: skel,
			root_defaults: root_skel,
		};
		if rig.height <= 0. {
			continue;
		};
		info!("adding humanoid rig to entity");
		cmds.entity(root_mesh).insert(rig);
	}
}

fn autoassign_from_vrm(
	root_mesh: Entity,
	vrm: &Vrm,
	children: &Query<&Children>,
	entity_names: &Query<(Entity, &Name)>,
) -> Result<HashMap<BoneKind, Entity>> {
	let (vrm_kind, nodes_result) =
		match (&vrm.extensions.vrmc_vrm, &vrm.extensions.vrm0) {
			(Some(_), Some(_)) => {
				return Err(eyre!(
					"both vrm0 and vrm1 extension data was present for entity
					{root_mesh:?}. Refusing to autoassign rig."
				));
			}
			(None, None) => {
				return Err(eyre!(
					"both vrm0 and vrm1 extension data was missing for entity
					{root_mesh:?}. Cannot autoassign rig."
				));
			}
			(Some(vrm1), None) => ("vrm1", nodes_from_vrm1(vrm1)),
			(None, Some(vrm0)) => ("vrm0", nodes_from_vrm0(vrm0)),
		};

	let nodes = match nodes_result {
		Err(err) => {
			return Err(eyre!(
				"Failed to determine rig assignment from {vrm_kind} for entity
				{root_mesh:?}: {err}"
			));
		}
		Ok(rig) => rig,
	};

	let mut found_bones: HashMap<BoneKind, Entity> = default();
	for (bone, idx) in nodes.0.into_iter() {
		let node_handle = &vrm.gltf.nodes[idx.0 as usize];
		let Some((node_name, _)) = vrm
			.gltf
			.named_nodes
			.iter()
			.find(|(_str, handle)| *handle == node_handle)
		else {
			panic!("no gltf node exists at the index {}", idx.0);
		};

		for child in children.iter_descendants(root_mesh) {
			if let Ok((entity, name)) = entity_names.get(child) {
				if name.as_str() == node_name {
					found_bones.insert(bone, entity);
				}
			}

		}

		/*let Some(entity) = entity_names.iter().find_map(|(entity, entity_name)| {
			(entity_name.as_str() == node_name).then_some(entity)
		}) else {
			panic!("no entity with the same name as the node ({node_name}) exists");
		};*/

		//found_bones.insert(bone, entity);
	}
	Ok(found_bones)
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
	root_mesh: Entity,
	children: &Query<&Children>,
	names: &Query<(Entity, &Name)>,
	str_buf: &mut Local<String>,
) -> Result<HashMap<BoneKind, Entity>> {
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
			if let Ok(kind) = guess_bone_kind(name.1.as_str(), str_buf, None) {
				map.insert(c, kind);
			}
		}
	}

	// Now that we have a map of entity -> BoneKind as a list of candidates,
	// pick winners and reverse the map to be BoneKind -> Entity.
	// TODO: Do something smarter than just picking the last map entry.
	let found_bones = map
		.into_iter()
		.map(|(entity, bone)| (bone, entity))
		.collect();
	Ok(found_bones)
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
	/// See also <https://github.com/vrm-c/vrm-specification/blob/57ba30bcb2638617d5ee655e35444caa299971bd/specification/VRMC_vrm-1.0/humanoid.md>
	#[derive(Reflect, Debug, Eq, PartialEq, Copy, Clone, Hash, Component)]
	pub enum BoneKind {
		Neck,
		Hips,
		Head,
		Spine,
		Chest,
		UpperChest,
		RightEye,
		LeftEye,
		RightShoulder,
		LeftShoulder,
		RightUpperArm,
		LeftUpperArm,
		RightLowerArm,
		LeftLowerArm,
		RightHand,
		LeftHand,
		RightUpperLeg,
		LeftUpperLeg,
		RightLowerLeg,
		LeftLowerLeg,
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
	// TODO: add more patterns
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
