#![allow(clippy::too_many_arguments)]

pub mod dev_tools;
pub mod humanoid;
pub mod macro_utils;
pub mod test_scaffold;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct Direction {
	pub forward: bool,
	pub back: bool,
	pub up: bool,
	pub down: bool,
	pub left: bool,
	pub right: bool,
}

impl Direction {
	pub fn is_some(&self) -> bool {
		self.forward || self.back || self.up || self.down || self.left || self.right
	}

	#[inline]
	pub fn is_none(&self) -> bool {
		!self.is_some()
	}
}
