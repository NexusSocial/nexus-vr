//! The networking crate. This is a separate crate to ensure that its contents are as
//! isolated from the client and server code as possible.

pub mod data_model;

pub mod client;
mod lightyear;
pub mod server;

pub use crate::client::ClientPlugin;
pub use crate::server::ServerPlugin;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transports {
	Udp,
}
pub use lightyear::Interpolated;