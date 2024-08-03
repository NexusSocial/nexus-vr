//! V1 of the API. This is subject to change until we commit to stability, after
//! which point any breaking changes will go in a V2 api.

use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

/// Router of API V1
pub fn router() -> Router {
	Router::new().route("/create", post(create))
}

async fn create(_pubkey: Json<JWK>) -> String {
	String::from("did:web:todo")
}

#[derive(Debug, Serialize, Deserialize)]
struct JWK;
