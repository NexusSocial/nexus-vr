//! A Decentralized Identifier (aka [DID][spec]), is a globally unique
//! identifier that provides a general purpose way of looking up public keys
//! associated with the globally unique identifier.
//!
//! This means that unlike a UUID, someone can prove that they own a DID, and
//! you can encrypt messages using DIDs! This makes DIDs strictly more useful
//! than traditional UUIDs as account identifiers and are very useful for
//! building federated or decentralized services.
//!
//! Unlike traditional centralized accounts, services that use DIDs give users
//! custody over their account identity. Authentication of users can happen
//! without the need for a centralized service or database. Instead, whoever
//! holds the private keys associated with a DID will be able to authenticate as
//! the account owner.
//!
//! This gives users the ability to maintain the same account handles/identities
//! across multiple separate services (or migrate homeservers in a federated
//! system) without having to create a new, different, account or identity each
//! time.
//!
//! [spec]: https://www.w3.org/TR/did-core/

#![forbid(unsafe_code)]

use std::str::FromStr;

pub mod methods;
pub mod uri;
pub mod utf8bytes;

pub trait Did: FromStr {
	fn uri(&self) -> self::uri::DidUri;
}
