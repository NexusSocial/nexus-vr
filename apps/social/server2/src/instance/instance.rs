//! Instances are not purely client or server authoritative. Instead, they follow
//! a "distributed simulation with authority" scheme. This is inspired by an excellent
//! [article][gaffer_vr] on networked physics in VR by GafferOnGames.
//!
//! [gaffer_vr]: https://gafferongames.com/post/networked_physics_in_virtual_reality

use std::collections::{BTreeSet, HashMap, HashSet};

use super::{ChannelFormat, ChannelId, ClientId};
#[derive(Debug, Default)]
pub struct Instance {
	channels: ChannelManager,
	sessions: HashMap<ClientId, SessionState>,
}

/// State associated with a client session.
#[derive(Debug)]
struct SessionState {
	/// All channels that this client has registered themselves on during this
	/// session.
	channels: HashMap<ChannelFormat, ChannelId>,
	/// The client associated with this session.
	client: ClientId,
}

/// Manages allocation and indices of channels.
#[derive(Debug, Default)]
struct ChannelManager {
	formats: HashMap<ChannelFormat, ChannelId>,
	/// Channels which have been deleted.
	holes: BTreeSet<ChannelId>,
	/// Indexed by `ChannelId`. Deleted channels will be noted in `holes`
	/// and must be re-initialized before use. They are not dropped when
	/// deleted to avoid memory allocator pressure.
	channels: Vec<Channel>,
}

impl ChannelManager {
	/// Registers a [`ChannelFormat`] and returns the [`ChannelId`] that references it.
	///
	/// If the channel already is registered, returns Err(ChannelId).
	fn register_channel(
		&mut self,
		format: ChannelFormat,
	) -> Result<ChannelId, ChannelId> {
		if let Some(&id) = self.formats.get(&format) {
			return Err(id);
		}
		// We use the first hole to prioiritize filling the values closer to the
		// start of the vec. This will help reduce fragmentation in `self.channels`.
		if let Some(hole) = self.holes.pop_first() {
			self.channels[usize::try_from(hole.0).unwrap()].reinitialize();
			Ok(hole)
		} else {
			self.channels.push(Channel::default());
			Ok(ChannelId((self.channels.len() - 1).try_into().unwrap()))
		}
	}
}

#[derive(Debug, Default)]
struct Channel {}

impl Channel {
	/// resets channel to default state, while reusing underlying memory buffers.
	fn reinitialize(&mut self) {}
}
