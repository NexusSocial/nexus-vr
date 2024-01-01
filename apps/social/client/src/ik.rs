//! Handles updated the transforms of humanoid rigs based on pose
#![allow(clippy::all)] // TODO: Un-allow this.

use crate::controllers::KeyboardController;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::trackers::OpenXRTrackingRoot;
use color_eyre::eyre;
use eyre::eyre;
use social_common::humanoid::{BoneKind, HumanoidRig, Skeleton, SKELETON_ARR_LEN};
use std::f32::consts::PI;
use social_networking::data_model::{Isometry, PlayerPose};
use crate::avatars::LocalEntity;

use bevy::prelude::*;

const DEFAULT_EYE_HEIGHT: f32 = 1.69 / 1.10; // slightly more than the height of the average American (men and women)
const HEURISTIC_NECK_PITCH_SCALE_CONSTANT: f32 = 0.7517 * PI; // ~135.3 degrees, from [1]
const HEURISTIC_NECK_PITCH_HEAD_CONSTANT: f32 = 0.333; // from [1]
const HEURISTIC_CHEST_PITCH_HEAD_CONSTANT: f32 = 0.05; // I made this up, based on observation that the chest responded less to head tilt
const HEURISTIC_SHOULDER_SCALING_CONSTANT: f32 = 0.167 * PI; // from [1]
const DEFAULT_SHOULDER_MOVEMENT_THRESHOLD: f32 = 0.5; // from [1]
const DEFAULT_SHOULDER_ROTATION_YAW_CONSTRAINT_MIN: f32 = 0.; // from [1], probably wouldn't hurt if it was a few degrees lower.
const DEFAULT_SHOULDER_ROTATION_YAW_CONSTRAINT_MAX: f32 = 0.183 * PI; // ~33 degrees, from [1]. Seems about right.
const DEFAULT_SHOULDER_ROTATION_ROLL_CONSTRAINT_MIN: f32 = 0.; // from [1], doesn't seem right these are the same
const DEFAULT_SHOULDER_ROTATION_ROLL_CONSTRAINT_MAX: f32 = 0.183 * PI; // from [1], doesn't seem right these are the same
const HEURISTIC_ELBOW_MODEL_BIASES: Vec3 =
	Vec3::new(30. * PI / 180., 120. * PI / 180., 65. * PI / 180.); // from [1], modified by myself, derived through manual optimization
const HEURISTIC_ELBOW_MODEL_WEIGHTS: Vec3 =
	Vec3::new(-50. * PI / 180., -60. * PI / 180., 260. * PI / 180.); // from [1], modified by myself, derived through manual optimization
const HEURISTIC_ELBOW_MODEL_OFFSET: f32 = 0.083 * PI; // ~15 degrees, from [1]
const HEURISTIC_ELBOW_MODEL_CONSTRAINT_MIN: f32 = 0.072 * PI; // ~13 degrees, from [1]
const HEURISTIC_ELBOW_MODEL_CONSTRAINT_MAX: f32 = 0.972 * PI; // ~175 degrees, from [1]
const HEURISTIC_ELBOW_SINGULARITY_RADIAL_THRESHOLD: f32 = 0.5; // from [1]
const HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MIN: f32 = 0.; // from [1]
const HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MAX: f32 = 0.1; // from [1]
const HEURISTIC_ELBOW_SINGULARITY_VECTOR: Vec3 = Vec3 {
	x: 0.133,
	y: -0.443,
	z: -0.886,
}; // from [1], probably normalized but I haven't checked; represents a direction.
   // to some extent it feels like the wrist yaw heuristics should come with constraints
const HEURISTIC_ELBOW_WRIST_YAW_THRESHOLD_LOWER: f32 = -0.25 * PI; // 45 degrees, from [1]
const HEURISTIC_ELBOW_WRIST_YAW_THRESHOLD_UPPER: f32 = 0.25 * PI; // from [1], mostly makes sense that these are the same.
const HEURISTIC_ELBOW_WRIST_YAW_SCALING_CONSTANT_LOWER: f32 = 1. / (-0.75 * PI); // 135^-1 degrees, from [1]. It makes sense in context.
const HEURISTIC_ELBOW_WRIST_YAW_SCALING_CONSTANT_UPPER: f32 = 1. / (0.75 * PI); // 135^-1 degrees, from [1]
const HEURISTIC_ELBOW_WRIST_ROLL_THRESHOLD_LOWER: f32 = 0.; // 0 degrees, from [1].
const HEURISTIC_ELBOW_WRIST_ROLL_THRESHOLD_UPPER: f32 = 0.5 * PI; // from [1]
const HEURISTIC_ELBOW_WRIST_ROLL_SCALING_CONSTANT_LOWER: f32 = 1. / (-(1. / 0.3) * PI); // 600^-1 degrees, from [1]
const HEURISTIC_ELBOW_WRIST_ROLL_SCALING_CONSTANT_UPPER: f32 = 1. / (1. / 0.6 * PI); // 300^-1 degrees, from [1]
const INCON_SKEL_ERR: &str = "wtf how, Skeleton modified in ways it shouldn't be";
const USE_ELBOW_HEURISTIC: bool = false;
const ELBOW_POLE_TARGET: Vec3 = Vec3::new(0.7, -0.5, -0.3);

#[derive(Default)]
pub struct IKPlugin;

impl Plugin for IKPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(
			PreUpdate,
			update_ik.run_if(bevy::ecs::prelude::resource_exists::<OculusController>())
		);
	}
}

fn iso_to_transform(iso: &Isometry) -> Transform {
	Transform {
		translation: iso.trans,
		rotation: iso.rot,
		scale: Vec3::ONE,
	}
}
// note: solves for the angle
fn cos_law(a: f32, b: f32, c: f32) -> f32 {
	// a+b>c must be true
	((a.powi(2) + b.powi(2) - c.powi(2)) / (2. * a * b)).acos()
}

fn orthonormalize(to_project: Vec3, fixed: Vec3) -> Vec3 {
	let normalized_fixed = fixed.normalize(); // Step 1
	let projection = to_project.dot(normalized_fixed) * normalized_fixed; // Step 2
	let orthonormal_vector = to_project - projection; // Step 3

	orthonormal_vector.normalize() // Step 4 (optional)
}

fn quat_from_vectors(plus_x: Vec3, plus_z: Vec3) -> Quat {
	let plus_y: Vec3 = plus_z.cross(plus_x);

	let mat: Mat3 = Mat3::from_cols(plus_x, plus_y, plus_z);

	// CARGO CULT ALERT: You probably shouldn't need to normalize here!
	Quat::from_mat3(&mat).normalize()
}

fn update_ik(
	input_query: Query<(&PlayerPose, &LocalEntity)>,
	local_local_avatar: Query<Entity, With<KeyboardController>>,
	avatar_children: Query<&Children>,
	skeleton_query: Query<&HumanoidRig>,
	mut transforms: Query<
		&mut Transform,
		(With<BoneKind>,
		Without<OpenXRTrackingRoot>)
	>,
	mut tracking_root: Query<&mut Transform, With<OpenXRTrackingRoot>>,
) {
	for (pose, loc_ent) in input_query.iter() {
		let mut func = || -> color_eyre::Result<()> {
			let local_local = local_local_avatar.contains(loc_ent.0);
			let mut skeleton_query_opt: Option<&HumanoidRig> = None;
			for child in avatar_children.iter_descendants(loc_ent.0) {
				if let Ok(vrm) = skeleton_query.get(child) {
					skeleton_query_opt = Some(vrm);
					break;
				}
			}
			// system should loop harmlessly until a SkeletonComponent is added
			let skeleton_query_out = match skeleton_query_opt {
				Some(q) => q,
				None => return Ok(()),
			};
			let skeleton_comp = skeleton_query_out;
			let height_factor = skeleton_comp.height / DEFAULT_EYE_HEIGHT;
			if local_local {
				for mut root in tracking_root.iter_mut() {
					root.scale = Vec3::ONE * height_factor;
				}
			}
			// read the state of the skeleton from the transforms
			let mut skeleton_transform_array: [Option<Transform>; SKELETON_ARR_LEN] =
				Default::default();
			let mut found_nan: bool = false;
			let test = true;
			fn contains_nan(input: Vec3) -> bool {
				input.x.is_nan() || input.y.is_nan() || input.z.is_nan()
			}
			fn quat_contains_nan(input: Quat) -> bool {
				input.x.is_nan() || input.y.is_nan() || input.z.is_nan() || input.w.is_nan()
			}
			for i in 0..SKELETON_ARR_LEN {
				let entity = match skeleton_comp.entities[i] {
					Some(e) => e,
					None => continue,
				};
				skeleton_transform_array[i] = Some(*transforms.get(entity)?);
				if test {
					match skeleton_transform_array[i] {
						Some(t) => {
							if contains_nan(t.translation) {
								error!("Incoming transform {:?} contained a NaN in translation", i);
								found_nan = true;
							}
							if quat_contains_nan(t.rotation) {
								error!(
									"Incoming transform {:?} contained a NaN in rotation",
									i
								);
								found_nan = true;
							}
							if contains_nan(t.scale) {
								error!(
									"Incoming transform {:?} contained a NaN in scale",
									i
								);
								found_nan = true;
							}
						}
						None => {}
					}
				}
			}
			if found_nan {
				error!("IK found a NaN, details precede. Returning to default skeleton and proceding.");
				skeleton_transform_array = skeleton_comp.defaults.arrayify();
			}
			let mut skeleton = Skeleton::new(skeleton_transform_array)?;
			// it's confusing to me that this is a z axis rotation? I would expect z axis to be roll. The forward vector of the hands must be fucky.
			let flip_quat_l = Quat::from_rotation_z(PI / 2.);
			let flip_quat_r = Quat::from_rotation_z(-PI / 2.);
			// At this point, the skeleton is known to be valid; next, handle VR input
			let final_head = iso_to_transform(&pose.head);
			let final_left_hand = iso_to_transform(&pose.hand_l);
			let final_right_hand = iso_to_transform(&pose.hand_r);
			// now everything is set up and IK logic can begin.
			let head_rot_euler = (skeleton_comp.root_defaults.head.rotation.inverse()
				* final_head.rotation)
				.to_euler(EulerRot::YXZ);
			let mut chest_pitch =
				(skeleton_comp.height - final_head.translation.y) / skeleton_comp.height;
			let mut neck_pitch = chest_pitch
				* (HEURISTIC_NECK_PITCH_SCALE_CONSTANT
					+ HEURISTIC_NECK_PITCH_HEAD_CONSTANT * head_rot_euler.1);
			chest_pitch *= HEURISTIC_NECK_PITCH_SCALE_CONSTANT
				+ HEURISTIC_CHEST_PITCH_HEAD_CONSTANT * head_rot_euler.1;
			neck_pitch -= chest_pitch; // Rotations propagate through transforms
			match skeleton.neck {
				Some(mut t) => {
					let neck_rot = Quat::from_euler(
						EulerRot::YXZ,
						0.,
						-neck_pitch.clamp(0., PI / 4.),
						0.,
					) * skeleton_comp
						.defaults
						.neck
						.ok_or(eyre!(INCON_SKEL_ERR))?
						.rotation;
					t.rotation = neck_rot.normalize();
				}
				None => {}
			}
			let head_flat = Vec2::new(final_head.translation.x, final_head.translation.z);
			let l_hand_flat =
				Vec2::new(final_left_hand.translation.x, final_left_hand.translation.z)
					- head_flat;
			let r_hand_flat = Vec2::new(
				final_right_hand.translation.x,
				final_right_hand.translation.z,
			) - head_flat;
			let mut hand_dir = l_hand_flat + r_hand_flat;
			let dir_len = hand_dir.length() / 0.2;
			if dir_len < 1. {
				hand_dir = Vec2::Y
					.rotate(l_hand_flat - r_hand_flat)
					.lerp(hand_dir, dir_len)
			}
			let head_rot = final_head.forward();
			let head_rot_flat = Vec2::new(head_rot.x, head_rot.z);
			if hand_dir.angle_between(head_rot_flat).abs() > PI / 2. {
				hand_dir *= -1.;
			}
			let dir_len = hand_dir.length() / 0.1;
			if dir_len < 1. {
				hand_dir = head_rot_flat.lerp(hand_dir, dir_len);
			}
			let chest_yaw = (hand_dir * -1.).angle_between(Vec2::Y);
			let new_hip_rot = Quat::from_euler(
				EulerRot::YXZ,
				chest_yaw,
				-chest_pitch.clamp(0., PI / 2.),
				0.,
			);
			skeleton.hips.rotation =
				(new_hip_rot * skeleton_comp.defaults.hips.rotation).normalize();
			let final_hands = [final_left_hand, final_right_hand];
			let mut skeleton_sides = [&mut skeleton.left, &mut skeleton.right];
			let default_skel_sides =
				[skeleton_comp.defaults.left, skeleton_comp.defaults.right];
			let root_default_skel_sides = [
				skeleton_comp.root_defaults.left,
				skeleton_comp.root_defaults.right,
			];
			let flip_quat_sides = [flip_quat_l, flip_quat_r];
			// return shoulders to neutral position
			for i in 0..2 {
				match skeleton_sides[i].shoulder {
					Some(mut t) => {
						t.rotation = default_skel_sides[i]
							.shoulder
							.ok_or(eyre!(INCON_SKEL_ERR))?
							.rotation
					}
					None => {}
				}
			}
			// calculate position of shoulders and head relative to root
			drop(skeleton_sides);
			skeleton.head.rotation = skeleton_comp.defaults.head.rotation;
			let mut root_skeleton = skeleton.recalculate_root(); // this function is likely quite expensive compared to everything else
			let new_offset = final_head.translation - root_skeleton.head.translation;
			skeleton.hips.translation = skeleton.hips.translation + new_offset;
			skeleton.head.rotation =
				(root_skeleton.head.rotation.inverse() * final_head.rotation).normalize();
			root_skeleton = skeleton.recalculate_root(); // this function is likely quite expensive compared to everything else
			skeleton_sides = [&mut skeleton.left, &mut skeleton.right];
			let mut root_skel_sides = [root_skeleton.left, root_skeleton.right];
			let mut had_shoulders = false;
			for i in 0..2 {
				// accounting for scaling is really annoying
				let upper_leg_length = (root_skel_sides[i].leg.lower.translation
					- root_skel_sides[i].leg.upper.translation)
					.length();
				let lower_leg_length = (root_skel_sides[i].foot.translation
					- root_skel_sides[i].leg.lower.translation)
					.length();
				// the spine is assumed to be very close to the center of mass
				let spine_offset = (root_skeleton.spine.translation
					- skeleton_comp.root_defaults.spine.translation)
					* Vec3::new(1., 0., 1.);
				let relevant_hip_rot = Quat::from_euler(EulerRot::YXZ, chest_yaw, 0., 0.);
				let target_foot_pos = relevant_hip_rot
					.mul_vec3(root_default_skel_sides[i].foot.translation)
					+ spine_offset;
				let target_foot_rot = (root_default_skel_sides[i].foot.rotation
					* relevant_hip_rot)
					.normalize();
				let target_foot = root_default_skel_sides[i]
					.foot
					.with_translation(target_foot_pos)
					.with_rotation(target_foot_rot);
				let mut foot_hip_vec =
					target_foot_pos - root_skel_sides[i].leg.upper.translation;
				let mut foot_hip_dist = foot_hip_vec.length();
				let combined_leg_length = (upper_leg_length + lower_leg_length) * 0.999;
				if foot_hip_dist > combined_leg_length {
					foot_hip_dist = combined_leg_length;
					foot_hip_vec = foot_hip_vec.normalize() * combined_leg_length
				}
				let minimum_combined_leg_length =
					(upper_leg_length - lower_leg_length).abs() * 1.0001;
				if foot_hip_dist < minimum_combined_leg_length {
					foot_hip_dist = minimum_combined_leg_length;
					foot_hip_vec = foot_hip_vec.normalize() * minimum_combined_leg_length
				}
				let foot_knee_angle =
					cos_law(upper_leg_length, foot_hip_dist, lower_leg_length);
				if foot_knee_angle.is_nan() {
					panic!("leg cos law failed")
				}
				let target_knee_pos = Quat::from_axis_angle(
					skeleton.hips.forward().cross(foot_hip_vec),
					-foot_knee_angle,
				)
				.mul_vec3(foot_hip_vec)
				.normalize();
				let uleg_rot = Quat::from_rotation_arc(
					skeleton_sides[i].leg.lower.translation.normalize(),
					target_knee_pos,
				);
				if uleg_rot.x.is_nan() {
					panic!("knee rotation calculation failed")
				}
				skeleton_sides[i].leg.upper.rotation =
					(skeleton.hips.rotation.inverse() * uleg_rot).normalize();
				let updated_root_upper_leg =
					root_skel_sides[i].leg.upper.with_rotation(uleg_rot);
				let curr_root_lower_leg = skeleton_sides[i]
					.leg
					.lower
					.with_rotation(Quat::IDENTITY)
					.mul_transform(updated_root_upper_leg);
				let target_foot_rel_knee = GlobalTransform::from(target_foot)
					.reparented_to(&GlobalTransform::from(curr_root_lower_leg));
				let lleg_rot = Quat::from_rotation_arc(
					skeleton_sides[i].foot.translation.normalize(),
					target_foot_rel_knee.translation.normalize(),
				);
				skeleton_sides[i].leg.lower.rotation =
					(updated_root_upper_leg.rotation.inverse() * lleg_rot).normalize();
				skeleton_sides[i].foot.rotation =
					(lleg_rot.inverse() * target_foot.rotation).normalize();
				match skeleton_sides[i].shoulder {
					Some(mut t) => {
						let x_side = i as f32 * -2. + 1.;
						let default_shoulder = default_skel_sides[i]
							.shoulder
							.ok_or(eyre!(INCON_SKEL_ERR))?;
						let shoulder_hand = root_skel_sides[i].arm.upper.translation
							- final_hands[i].translation;
						let calc_shoulder_angle = |input| -> f32 {
							HEURISTIC_SHOULDER_SCALING_CONSTANT
								* (shoulder_hand.dot(input)
									/ (skeleton_sides[i].arm.lower.translation.length()
										+ skeleton_sides[i].hand.translation.length())
									- DEFAULT_SHOULDER_MOVEMENT_THRESHOLD)
						};
						let mut shoulder_yaw =
							calc_shoulder_angle(root_skeleton.spine.forward());
						shoulder_yaw = shoulder_yaw.clamp(
							DEFAULT_SHOULDER_ROTATION_YAW_CONSTRAINT_MIN,
							DEFAULT_SHOULDER_ROTATION_YAW_CONSTRAINT_MAX,
						) * x_side;
						let mut shoulder_roll =
							calc_shoulder_angle(root_skeleton.spine.up());
						shoulder_roll = shoulder_roll.clamp(
							DEFAULT_SHOULDER_ROTATION_ROLL_CONSTRAINT_MIN,
							DEFAULT_SHOULDER_ROTATION_ROLL_CONSTRAINT_MAX,
						);
						let shoulder_rot = Quat::from_euler(
							EulerRot::YXZ,
							shoulder_yaw,
							0.,
							shoulder_roll,
						) * default_shoulder.rotation;
						t.rotation = shoulder_rot.normalize();
						had_shoulders = true;
					}
					None => {}
				};
			}
			if had_shoulders {
				drop(skeleton_sides);
				root_skeleton = skeleton.recalculate_root();
				skeleton_sides = [&mut skeleton.left, &mut skeleton.right];
				root_skel_sides = [root_skeleton.left, root_skeleton.right];
			}
			let highest_root = match root_skeleton.upper_chest {
				Some(t) => t,
				None => match root_skeleton.chest {
					Some(t) => t,
					None => root_skeleton.spine,
				},
			};
			for i in 0..2 {
				let arm_highest_root = match root_skel_sides[i].shoulder {
					Some(t) => t,
					None => highest_root,
				};
				let up = highest_root.up();
				let forward = highest_root.forward();
				let x_side = i as f32 * -2. + 1.;
				// only vaguely shoulder-like. It is, at least, in the right place.
				let virtual_shoulder = root_skel_sides[i]
					.arm
					.upper
					.looking_to(forward, up)
					.with_scale(Vec3::ONE);
				// I currently am not confident the following elbow code works correctly
				let mut final_hand_rel_v_shoulder = GlobalTransform::from(final_hands[i])
					.reparented_to(&GlobalTransform::from(virtual_shoulder))
					.translation;
				// accounting for scaling is fucking annoying, but technically the VRM spec allows it!
				let upper_arm_length = (root_skel_sides[i].arm.lower.translation
					- root_skel_sides[i].arm.upper.translation)
					.length();
				let forearm_length = (root_skel_sides[i].hand.translation
					- root_skel_sides[i].arm.lower.translation)
					.length();
				let mut shoulder_hand_length = final_hand_rel_v_shoulder.length();
				// don't want the game to crash if the player's hand gets slightly further than expected from their body
				let combined_arm_length = (upper_arm_length + forearm_length) * 0.999;
				if shoulder_hand_length > combined_arm_length {
					shoulder_hand_length = combined_arm_length;
					final_hand_rel_v_shoulder =
						final_hand_rel_v_shoulder.normalize() * combined_arm_length
				}
				// Avatars can have somewhat weird proportions that prevent them from touching their shoulders
				let minimum_combined_arm_length =
					(upper_arm_length - forearm_length).abs() * 1.0001;
				if shoulder_hand_length < minimum_combined_arm_length {
					shoulder_hand_length = minimum_combined_arm_length;
					final_hand_rel_v_shoulder =
						final_hand_rel_v_shoulder.normalize() * minimum_combined_arm_length;
				}
				// these next two lines should account for scale, probably?
				let mirror_vec = Vec3::new(0., 1., 1.) + Vec3::X * x_side * -1.;
				let mut elbow_vec: Vec3;
				let final_hand_rel_shoulder =
					final_hands[i].translation - root_skel_sides[i].arm.upper.translation;
				if USE_ELBOW_HEURISTIC {
					// the coordinate system is unspecified in the paper afaik, I can only pray this works ig
					let model_in =
						final_hand_rel_v_shoulder * mirror_vec / skeleton_comp.height;
					let model_out = model_in
						.mul_add(
							HEURISTIC_ELBOW_MODEL_WEIGHTS,
							HEURISTIC_ELBOW_MODEL_BIASES,
						)
						.clamp(Vec3::ZERO, Vec3::INFINITY)
						.dot(Vec3::ONE);
					let elbow_roll = (HEURISTIC_ELBOW_MODEL_OFFSET + model_out).clamp(
						HEURISTIC_ELBOW_MODEL_CONSTRAINT_MIN,
						HEURISTIC_ELBOW_MODEL_CONSTRAINT_MAX,
					);
					elbow_vec =
						Quat::from_axis_angle(final_hand_rel_v_shoulder, elbow_roll)
							.mul_vec3(Vec3::Y);
					let hand_dist_thresh =
						HEURISTIC_ELBOW_SINGULARITY_RADIAL_THRESHOLD * height_factor;
					let hand_horiz_dist =
						(final_hand_rel_v_shoulder * Vec3::new(1., 0., 1.)).length();
					let hand_dist_scaled = hand_horiz_dist / hand_dist_thresh;
					if hand_dist_scaled < 1. {
						elbow_vec = HEURISTIC_ELBOW_SINGULARITY_VECTOR
							.lerp(elbow_vec, hand_dist_scaled)
					}
					if final_hand_rel_v_shoulder.z
						> HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MIN
						&& final_hand_rel_v_shoulder.z
							< HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MAX
					{
						let a = HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MIN;
						let b =
							(HEURISTIC_ELBOW_SINGULARITY_FORWARD_THRESHOLD_MAX - a) / 2.;
						let lerp_amount = (final_hand_rel_v_shoulder.z - a + b) / b; // should be 0 at the center, and 1 at the edges
						elbow_vec =
							HEURISTIC_ELBOW_SINGULARITY_VECTOR.lerp(elbow_vec, lerp_amount);
					}
					let hand_angle_in_arm = virtual_shoulder
						.looking_to(final_hand_rel_v_shoulder, elbow_vec)
						.rotation
						.inverse() * final_hands[i].rotation;
					let hand_angle_euler = hand_angle_in_arm.to_euler(EulerRot::YXZ);
					let hand_yaw = hand_angle_euler.0;
					let hand_roll = hand_angle_euler.2;
					let hand_yaw_thresh_over =
						hand_yaw - HEURISTIC_ELBOW_WRIST_YAW_THRESHOLD_UPPER;
					let hand_yaw_thresh_under =
						hand_yaw - HEURISTIC_ELBOW_WRIST_YAW_THRESHOLD_LOWER;
					let hand_roll_thresh_over =
						hand_roll - HEURISTIC_ELBOW_WRIST_ROLL_THRESHOLD_UPPER;
					let hand_roll_thresh_under =
						hand_roll - HEURISTIC_ELBOW_WRIST_ROLL_THRESHOLD_LOWER;
					let mut hand_roll_correction: f32 = 0.;
					if hand_yaw_thresh_over > 0. {
						hand_roll_correction += hand_yaw_thresh_over.powi(2)
							* HEURISTIC_ELBOW_WRIST_YAW_SCALING_CONSTANT_UPPER;
					}
					if hand_yaw_thresh_under < 0. {
						hand_roll_correction += hand_yaw_thresh_under.powi(2)
							* HEURISTIC_ELBOW_WRIST_YAW_SCALING_CONSTANT_LOWER;
					}
					if hand_roll_thresh_over > 0. {
						hand_roll_correction += hand_roll_thresh_over.powi(2)
							* HEURISTIC_ELBOW_WRIST_ROLL_SCALING_CONSTANT_UPPER;
					}
					if hand_roll_thresh_under < 0. {
						hand_roll_correction += hand_roll_thresh_under.powi(2)
							* HEURISTIC_ELBOW_WRIST_ROLL_SCALING_CONSTANT_LOWER;
					}
	
					elbow_vec = Quat::from_axis_angle(
						final_hand_rel_v_shoulder.normalize(),
						hand_roll_correction,
					)
					.mul_vec3(elbow_vec);
					// convert the elbow vec from the virtual shoulder coordinate system to the root coordinate system
					// the elbow vec represents an orientation, so this purely involves rotation
					elbow_vec = virtual_shoulder.rotation.inverse().mul_vec3(elbow_vec);
				} else {
					elbow_vec = skeleton.hips.rotation
						* (ELBOW_POLE_TARGET * mirror_vec * skeleton_comp.height)
						- virtual_shoulder.translation;
				}
				let hand_upper_arm_angle =
					cos_law(upper_arm_length, shoulder_hand_length, forearm_length);
				if hand_upper_arm_angle.is_nan() {
					panic!(
						"arm cos law failed. (a, b, c) = {:?}",
						(upper_arm_length, shoulder_hand_length, forearm_length)
					)
				}
				let elbow_axis = final_hand_rel_shoulder
					.normalize()
					.cross(elbow_vec.normalize())
					.normalize();
				let hand_upper_arm_rot =
					Quat::from_axis_angle(elbow_axis, hand_upper_arm_angle).normalize();
				let new_elbow_pos = hand_upper_arm_rot
					.mul_vec3(final_hand_rel_shoulder.normalize() * upper_arm_length);
				// new new algorithm; do the roll seperately, by mapping (-1, 1, 0)
				// calculation of the base rotation could theoretically be calculated at setup time
				let uarm_rot_base = Quat::from_rotation_arc(
					skeleton_sides[i].arm.lower.translation.normalize(),
					Vec3::Z,
				);
				// I am very concerned about why this doesn't work as a rotation arc
				let uarm_plus_z = new_elbow_pos.normalize();
				let uarm_plus_x = orthonormalize(elbow_axis, uarm_plus_z);
				let uarm_rot_final = quat_from_vectors(uarm_plus_x, uarm_plus_z);
				let new_upper_arm_rot = (uarm_rot_final * uarm_rot_base).normalize();
				skeleton_sides[i].arm.upper.rotation =
					(arm_highest_root.rotation.inverse() * new_upper_arm_rot).normalize();
				// next line is certified correct
				let updated_root_upper_arm =
					arm_highest_root.mul_transform(skeleton_sides[i].arm.upper);
				let curr_root_lower_arm = updated_root_upper_arm
					.mul_transform(skeleton_sides[i].arm.lower)
					.translation;
				//let mut dislocation = curr_root_lower_arm - (new_elbow_pos*upper_arm_length + updated_root_upper_arm.translation);
				//if dislocation.length() > 0.01 {panic!("Ur arm math broke lol, dislocation  = {:?}", dislocation)}
				//println!("Checking things: {:?}", (curr_root_lower_arm.translation, updated_root_upper_arm.translation, new_elbow_pos*upper_arm_length + updated_root_upper_arm.translation));
				let target_hand_rel_elbow =
					final_hands[i].translation - curr_root_lower_arm;
				let larm_rot_base = Quat::from_rotation_arc(
					skeleton_sides[i].hand.translation.normalize(),
					Vec3::Z,
				);
	
				// let hand_plus_z: Vec3 = final_hands[i].rotation * Vec3::Z;
				// let hand_plus_x: Vec3 = (final_hands[i].rotation * Vec3::X).normalize();
				// x_sidez
				let hand_plus_x: Vec3 =
					(final_hands[i].rotation * Vec3::Z * x_side).normalize();
				let forearm_plus_z: Vec3 = (target_hand_rel_elbow).normalize();
	
				let forearm_plus_x: Vec3 = orthonormalize(hand_plus_x, forearm_plus_z);
	
				let forearm_rotation_global: Quat =
					quat_from_vectors(forearm_plus_x, forearm_plus_z);
	
				let new_lower_arm_rot =
					(forearm_rotation_global * larm_rot_base).normalize();
				//if new_lower_arm_rot.x.is_nan() {panic!("lower arm rotation calculation failed")}
				skeleton_sides[i].arm.lower.rotation =
					(updated_root_upper_arm.rotation.inverse() * new_lower_arm_rot)
						.normalize();
				let updated_root_lower_arm =
					updated_root_upper_arm.mul_transform(skeleton_sides[i].arm.lower);
				let curr_root_hand = updated_root_lower_arm.mul_transform(
					skeleton_sides[i]
						.hand
						.with_rotation(default_skel_sides[i].hand.rotation),
				);
				//dislocation = curr_root_hand.translation - (target_hand_rel_elbow.normalize()*forearm_length + updated_root_lower_arm.translation);
				/*if dislocation.length() > 0.05 {
					panic!("Ur hand math (3) broke lol, dislocation  = {:?}", dislocation)
				}
				dislocation = final_hands[i].translation - (target_hand_rel_elbow + updated_root_lower_arm.translation);
				if dislocation.length() > 0.05 {
					panic!("Ur hand math (2) broke lol, dislocation  = {:?}", dislocation)
				}*/
				skeleton_sides[i].hand.rotation = (curr_root_hand.rotation.inverse()
					* (final_hands[i].rotation * flip_quat_sides[i]))
					.normalize();
				// WOOOO!!!! OH YEAH BABY THAT'S EVERYTHING!!!!
			}
			drop(skeleton_sides);
			// write the changes to skeleton back to the transforms
			let skeleton_array = skeleton.arrayify();
			for i in 0..SKELETON_ARR_LEN {
				let entity = match skeleton_comp.entities[i] {
					Some(e) => e,
					None => continue,
				};
				let bone = skeleton_array[i].ok_or(eyre!(INCON_SKEL_ERR))?;
				*transforms.get_mut(entity)? = bone;
				if test {
					match skeleton_array[i] {
						Some(t) => {
							if contains_nan(t.translation) {
								error!("Outgoing transform {:?} contained a NaN in translation", i);
								found_nan = true;
							}
							if quat_contains_nan(t.rotation) {
								error!(
									"Outgoing transform {:?} contained a NaN in rotation",
									i
								);
								found_nan = true;
							}
							if contains_nan(t.scale) {
								error!(
									"Outgoing transform {:?} contained a NaN in scale",
									i
								);
								found_nan = true;
							}
						}
						None => {}
					}
				}
			}
			if found_nan {
				error!("Found a NaN, details precede. Exiting.");
				return Err(eyre!("Found a NaN, details precede. Exiting"));
			}
			Ok(())
		};
		let _ = func();
	};
}
