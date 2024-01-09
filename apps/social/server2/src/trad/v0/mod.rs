//! v0 of the HTTP api.

// Tip: add `#[axum_macros::debug_handler]` to see better errors when a handler
// has a compile error.

use axum::routing::post;
use axum_macros::debug_handler;

pub fn router() -> axum::Router {
	axum::Router::new().route("/instances/create", post(instances_create))
}

#[debug_handler]
async fn instances_create() {}
