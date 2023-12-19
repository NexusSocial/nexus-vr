//! Common functionality shared between client and server.
//!
//! This crate has the following goals:
//! - Zero graphics, physics, or networking dependencies (it should be headless). If
//!   you do need to pull in such a dependency for the purpose of implementing a trait,
//!   cfg gate that dependency and the trait impl.
//! - All systems should make no assumptions about whether a server or clients
//!   exist or not.
//! - IO (both network and filesystem) should be avoided. Try to defer the responsiblity
//!   of IO to the consumers of this crate. This helps make testing and running with
//!   atypical networking configurations easier.
//! - Let systems and components that are specific to the client or server be
//!   implemented by the client or server - this is not the home for them. Instead
//!   the client/server can introduce new additional systems/components that build on
//!   top of the functionality in this crate.
