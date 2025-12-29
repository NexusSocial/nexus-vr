//! The different types of ui views.
//!
//! Views are different, distinct layouts of ui for various use cases.

mod login;

use bevy::app::App;

pub use self::login::LoginUi;

pub(crate) fn add_systems(app: &mut App) {
	self::login::add_systems(app);
}
