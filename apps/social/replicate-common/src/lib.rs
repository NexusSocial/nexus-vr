pub mod did;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! make_uuid {
    {$(
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident;
    )*} => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
        $vis struct $ident(Uuid);

        impl $ident {
            pub fn random() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn into_uuid(self) -> Uuid {
                self.0
            }
        }
    )*}
}

make_uuid! {
	/// Identifies an instance. Globally unique.
	pub struct InstanceId;

	/// Identifies a client. Globally unique.
	pub struct ClientId;
}

/// Identifies a channel. Unique within a client's session.
#[derive(
	Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct ChannelId(pub u32);

/// Machine readable identifier or schema that describes the serialization format of
/// all messages on the channel.
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct ChannelFormat(pub Bytes);
