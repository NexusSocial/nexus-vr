pub mod key;
pub mod web;

/// Dynamically typed did method.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
#[non_exhaustive]
pub enum DidDyn {
	Key(self::key::DidKey),
	Web(self::web::DidWeb),
}
