use bevy::{
	prelude::{Component, Quat, Vec3},
	reflect::Reflect,
};
use lightyear::prelude::Message;
use lightyear::client::components::LerpFn;
use serde::{Deserialize, Serialize};

#[derive(
	Component,
	Message,
	Serialize,
	Deserialize,
	Clone,
	Copy,
	Debug,
	PartialEq,
	Default,
	Reflect,
)]
pub struct PlayerPose {
	/// The root of the avatar. Everything else is relative to this.
	// TODO: Change to Transform and remove seperate scale
	pub root: Isometry,
	pub root_scale: Vec3,
	pub head: Isometry,
	pub hand_l: Isometry,
	pub hand_r: Isometry,
}

/// An isometry is like a `Transform`, but without scale.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default, Reflect)]
pub struct Isometry {
	pub trans: Vec3,
	pub rot: Quat,
}

impl Isometry {
	pub fn mul_isometry(&self, other: Isometry) -> Isometry {
		Isometry {
			trans: self.trans + (self.rot * other.trans),
			rot: self.rot * other.rot,
		}
	}
	pub fn scale_translation(&self, scale: Vec3) -> Isometry {
		Isometry {
			trans: self.trans * scale,
			rot: self.rot,
		}
	}
}

/// Interpolates between two [`PlayerPose`]s.
pub struct PlayerPoseInterp;

impl LerpFn<PlayerPose> for PlayerPoseInterp {
	fn lerp(start: &PlayerPose, other: &PlayerPose, t: f32) -> PlayerPose {
		let mut start = *start;
		/// Mutates `start` for every specified field.
		macro_rules! call_interp {
			($start:ident, $other:ident, $t:ident, [$($field:ident),+]) => {
                $(
                    $start.$field = IsometryInterp::lerp(&$start.$field, &$other.$field, $t);
                )+
            };
		}
		call_interp!(start, other, t, [root, head, hand_l, hand_r]);

		start
	}
}

/// Interpolates between two [`Isometry`]s.
struct IsometryInterp;
impl LerpFn<Isometry> for IsometryInterp {
	fn lerp(start: &Isometry, other: &Isometry, t: f32) -> Isometry {
		let mut start = *start;
		start.trans = start.trans.lerp(other.trans, t);
		start.rot = start.rot.lerp(other.rot, t);

		start
	}
}
