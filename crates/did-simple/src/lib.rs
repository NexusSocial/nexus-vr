//! A Decentralized Identifier (aka [DID][spec]), is a globally unique
//! identifier that provides a general purpose way of looking up public keys
//! associated with the identifier.
//!
//! This means that ownership of a UUID can be proven and that support for
//! common cryptography operations such as signing, verifying, and encrypting
//! messages is possible. This makes DIDs strictly more useful than traditional
//! UUIDs as account identifiers and are very useful for building federated or
//! decentralized services.
//!
//! Services that use DIDs give users self-custody over their account identity.
//! Authentication of users can happen without the need for a centralized
//! authentication service or user database. Instead, whoever holds the private
//! keys associated with a DID will be able to authenticate as the account owner.
//!
//! This gives users the ability to maintain the same account handles/identities
//! across multiple separate services (or migrate homeservers in a federated
//! system) without having to create a different account or identity for each
//! service.
//!
//! [spec]: https://www.w3.org/TR/did-core/

#![forbid(unsafe_code)]

use std::str::FromStr;

pub(crate) mod key_algos;
pub mod methods;
pub mod url;
pub mod utf8bytes;
mod varint;

pub use crate::key_algos::KeyAlgo;
pub use crate::methods::DidDyn;
pub use crate::url::DidUrl;

pub trait Did: FromStr {
	fn url(&self) -> self::url::DidUrl;
}
