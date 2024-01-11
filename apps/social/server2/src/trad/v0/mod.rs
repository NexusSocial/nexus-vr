//! v0 of the HTTP api.

// Tip: add `#[axum_macros::debug_handler]` to see better errors when a handler
// has a compile error.

use axum::{extract::State, routing::post, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use crate::instance_manager::{self, InstanceId};

#[derive(Debug, Clone)]
pub(crate) struct ApiState {
	pub instance_manager: instance_manager::Handle,
}

pub fn router() -> axum::Router<ApiState> {
	axum::Router::new().route("/instances/create", post(instances_create))
}

#[derive(Deserialize, Serialize)]
struct InstancesCreateResponse {
	id: InstanceId,
}

#[debug_handler]
#[tracing::instrument]
async fn instances_create(
	State(state): State<ApiState>,
) -> Json<InstancesCreateResponse> {
	let id = state.instance_manager.new_instance().await;
	Json(InstancesCreateResponse { id })
}
