// TODO: Remove this
#![allow(unused)]

use dashmap::DashMap;
use replicate_common::InstanceId;
use tracing::debug;

use super::DataModel;

#[derive(Default, Debug)]
pub struct InstanceManager {
	instances: DashMap<InstanceId, DataModel>,
}

impl InstanceManager {
	pub fn instance_create(&self) -> InstanceId {
		let instance = DataModel::default();
		// TODO: seed random numbers for determinism?
		let id = InstanceId::random();
		self.instances.insert(id, instance);
		debug!("created instance {id:?}");
		id
	}
}
