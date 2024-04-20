//! Instances are not purely client or server authoritative. Instead, they follow
//! a "distributed simulation with authority" scheme. This is inspired by an excellent
//! [article][gaffer_vr] on networked physics in VR by GafferOnGames.
//!
//! [gaffer_vr]: https://gafferongames.com/post/networked_physics_in_virtual_reality

mod manager;

pub use self::manager::InstanceManager;

#[derive(Debug, Default)]
pub struct Instance {}
