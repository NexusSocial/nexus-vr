//! For now, we use serde for messaging because I'm trying to go fast.
//! We should switch to protobuf or capnproto as soon as we prove the networking
//! works.

pub mod manager;

mod framed;

pub use self::framed::Framed;
