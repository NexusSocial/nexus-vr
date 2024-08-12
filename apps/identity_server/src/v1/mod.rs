//! V1 of the API. This is subject to change until we commit to stability, after
//! which point any breaking changes will go in a V2 api.

use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Redirect},
	routing::{get, post},
	Json, Router,
};
use color_eyre::eyre::Context as _;
use jose_jwk::{Jwk, JwkSet};
use uuid::Uuid;

use crate::uuid::UuidProvider;

#[derive(Debug, Clone)]
struct RouterState {
	uuid_provider: Arc<UuidProvider>,
	db_pool: sqlx::sqlite::SqlitePool,
}

/// Configuration for the V1 api's router.
#[derive(Debug, Default)]
pub struct RouterConfig {
	pub uuid_provider: UuidProvider,
	pub db_pool_opts: sqlx::sqlite::SqlitePoolOptions,
	pub db_url: String,
}

impl RouterConfig {
	pub async fn build(self) -> color_eyre::Result<Router> {
		let db_pool = self
			.db_pool_opts
			.connect(&self.db_url)
			.await
			.wrap_err_with(|| {
				format!("failed to connect to pool with url {}", self.db_url)
			})?;

		sqlx::migrate!("./migrations")
			.run(&db_pool)
			.await
			.wrap_err("failed to run migrations")?;

		Ok(Router::new()
			.route("/create", post(create))
			.route("/users/:id/did.json", get(read))
			.with_state(RouterState {
				uuid_provider: Arc::new(self.uuid_provider),
				db_pool,
			}))
	}
}
async fn create(state: State<RouterState>, _pubkey: Json<Jwk>) -> Redirect {
	let uuid = state.uuid_provider.next_v4();
	Redirect::to(&format!("/users/{}/did.json", uuid.as_hyphenated()))
}

#[derive(thiserror::Error, Debug)]
enum ReadErr {
	#[error("no such user exists")]
	NoSuchUser,
	#[error(transparent)]
	Internal(#[from] color_eyre::Report),
}

impl IntoResponse for ReadErr {
	fn into_response(self) -> axum::response::Response {
		match self {
			ReadErr::NoSuchUser => {
				(StatusCode::NOT_FOUND, self.to_string()).into_response()
			}
			Self::Internal(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
			}
		}
	}
}

async fn read(
	state: State<RouterState>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<JwkSet>, ReadErr> {
	let keyset_in_string: Option<String> =
		sqlx::query_scalar("SELECT pubkeys FROM users WHERE user_id = $1")
			.bind(user_id)
			.fetch_optional(&state.db_pool)
			.await
			.wrap_err("failed to retrieve from database")?;
	let Some(keyset_in_string) = keyset_in_string else {
		return Err(ReadErr::NoSuchUser);
	};
	// TODO: Do we actually care about round-trip validating the JwkSet here?
	let keyset: JwkSet = serde_json::from_str(&keyset_in_string)
		.wrap_err("failed to deserialize JwkSet from database")?;
	Ok(Json(keyset))
}
