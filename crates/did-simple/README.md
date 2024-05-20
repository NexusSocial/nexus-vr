# did-simple

Dead simple [DID][spec]s (Decentralized Identifiers).

This crate provides the ability to:
* Parse did urls.
* (optional feature) Perform cryptographic operations on the public keys
  resolved from DIDs.

We intentionally do not perform IO, such as what would be required to resolve a
did:web. It is *your* responsibility to do IO, and then use this crate to
validate that data and get back something that you can do cryptographic operations
with. This ensures that this crate stays small, and that did-simple can be used
with any backend or client and in both sync and async paradigms.

Supported DID methods:
* did:key
* (coming soon) did:web

# Security

This crate enforces `#![forbid(unsafe_code)]` unless the `allow-unsafe` feature
is enabled. Since features in rust are additive across a dependency graph, don't
enable this feature unless you are writing an application!

This crate has a very high bar for the addition of new dependencies, because
dependencies are places where the software supply chain can be attacked. Right now,
we have zero non-rust dependencies, and passing `no-default-features` gives you
dependencies on only the following crates:

* thiserror (proc macro)
* bytes (no transitive deps)
* bs58 (no transitive deps)

We also test effectively every possible bit pattern when encoding and decoding
varints, a necessary part of did:key resolution.

# Breaking Changes

This crate is v0.0.X, and may introduce breaking changes at any time, with any
frequency.

[spec]: https://www.w3.org/TR/did-core/
