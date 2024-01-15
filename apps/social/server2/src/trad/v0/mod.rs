//! v0 of the HTTP api.

// Tip: add `#[axum_macros::debug_handler]` to see better errors when a handler
// has a compile error.

use std::sync::Arc;

#[allow(unused_imports)]
use axum_macros::debug_handler;

use axum::{extract::State, routing::post};

use crate::instance::InstanceManager;

#[derive(Debug, Clone)]
pub(crate) struct ApiState {
	pub(super) im: Arc<InstanceManager>,
}

pub fn router() -> axum::Router<ApiState> {
	axum::Router::new().route("/instances/create", post(instances_create))
}

#[tracing::instrument]
async fn instances_create(State(state): State<ApiState>) -> String {
	use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
	let id = state.im.instance_create();
	BASE64_URL_SAFE_NO_PAD.encode(id.into_uuid().into_bytes())
}
