//! A crate to manipulate DID chains.
//!
//! For more info on what a Decentralized Identifier aka "DID" is, you can
//! [read the spec][spec], or read the docs of the [`did_simple`] crate. The
//! TLDR is that you can treat it like a UUID, except that it also supports
//! signing and encrypting messages, as well as proving that you own the DID
//! without relying on any single centralized service.
//!
//! This crate builds upon the concept of a DID to introduce a chain of DIDs.
//! A did chain is a linear list of DIDs, starting with a root did. Each did in
//! the chain signs a message linking it to the next one in the chain. You then
//! can use the public keys of the last DID in the chain to get public keys
//! from, which may be significantly more convenient to use than the root DID.
//!
//! This allows end users to mix and match did methods, giving them the ability
//! to pick the right balance of convenience vs security for their needs. For
//! example, a user's root did:key could have its private keys live in cold
//! storage, and instead they do day to day signing with a DID:web that is
//! hosted by a third party. If they ever need to change their DID:web, they
//! can retrieve the root did and sign a message to migrate to a new child DID.
//!
//! [spec]: https://www.w3.org/TR/did-core/

#![forbid(unsafe_code)]

pub use did_simple;

use did_simple::{methods::key::DidKey, methods::DidDyn};

/// This is like an account UUID, it provides a unique identifier for the
/// account. Changing it is impossible.
#[derive(Debug)]
pub struct DidRoot(pub DidKey);

#[derive(Debug)]
pub struct DidChain {
	pub root: DidRoot,
	pub chain: Vec<DidDyn>,
}
