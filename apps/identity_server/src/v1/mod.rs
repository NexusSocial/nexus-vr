//! V1 of the API. This is subject to change until we commit to stability, after
//! which point any breaking changes will go in a V2 api.

use std::sync::Arc;

use axum::{
	extract::{NestedPath, Path, State},
	http::StatusCode,
	response::{IntoResponse, Redirect},
	routing::{get, post},
	Json, Router,
};
use color_eyre::eyre::Context as _;
use jose_jwk::{Jwk, JwkSet};
use tracing::error;
use uuid::Uuid;

use crate::{uuid::UuidProvider, MigratedDbPool};

#[derive(Debug, Clone)]
struct RouterState {
	uuid_provider: Arc<UuidProvider>,
	db_pool: MigratedDbPool,
}

/// Configuration for the V1 api's router.
#[derive(Debug)]
pub struct RouterConfig {
	pub uuid_provider: UuidProvider,
	pub db_pool: MigratedDbPool,
}

impl RouterConfig {
	pub async fn build(self) -> color_eyre::Result<Router> {
		Ok(Router::new()
			.route("/create", post(create))
			.route("/users/:id/did.json", get(read))
			.with_state(RouterState {
				uuid_provider: Arc::new(self.uuid_provider),
				db_pool: self.db_pool,
			}))
	}
}

#[derive(thiserror::Error, Debug)]
enum CreateErr {
	#[error(transparent)]
	Internal(#[from] color_eyre::Report),
}

impl IntoResponse for CreateErr {
	fn into_response(self) -> axum::response::Response {
		error!("{self:?}");
		match self {
			Self::Internal(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
			}
		}
	}
}

#[tracing::instrument(skip_all)]
async fn create(
	state: State<RouterState>,
	nested_path: NestedPath,
	pubkey: Json<Jwk>,
) -> Result<Redirect, CreateErr> {
	let uuid = state.uuid_provider.next_v4();
	let jwks = JwkSet {
		keys: vec![pubkey.0],
	};
	let serialized_jwks = serde_json::to_string(&jwks).expect("infallible");

	sqlx::query("INSERT INTO users (user_id, pubkeys_jwks) VALUES ($1, $2)")
		.bind(uuid)
		.bind(serialized_jwks)
		.execute(&state.db_pool.0)
		.await
		.wrap_err("failed to insert identity into db")?;

	Ok(Redirect::to(&format!(
		"{}/users/{}/did.json",
		nested_path.as_str(),
		uuid.as_hyphenated()
	)))
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
		error!("{self:?}");
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

// TODO: currently this returns a JSON Web Key Set, but we actually want to be
// returning a did:web json.
#[tracing::instrument(skip_all)]
async fn read(
	state: State<RouterState>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<JwkSet>, ReadErr> {
	let keyset_in_string: Option<String> =
		sqlx::query_scalar("SELECT pubkeys_jwks FROM users WHERE user_id = $1")
			.bind(user_id)
			.fetch_optional(&state.db_pool.0)
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

#[cfg(test)]
mod tests {
	use super::*;
	use axum::{
		body::Body,
		http::{Request, Response},
	};
	use color_eyre::Result;
	use http_body_util::BodyExt;
	use jose_jwk::OkpCurves;
	use sqlx::SqlitePool;
	use tower::ServiceExt as _; // for `collect`

	fn uuids(num_uuids: usize) -> Vec<Uuid> {
		(1..=num_uuids)
			.map(|x| Uuid::from_u128(x.try_into().unwrap()))
			.collect()
	}

	async fn test_router(db_pool: SqlitePool) -> Result<Router> {
		let db_pool = crate::MigratedDbPool::new(db_pool)
			.await
			.wrap_err("failed to migrate db")?;
		let router = RouterConfig {
			uuid_provider: UuidProvider::new_from_sequence(uuids(10)),
			db_pool,
		};
		router.build().await.wrap_err("failed to build router")
	}

	/// Validates the response and ensures it matches `expected_keys`
	async fn check_response_keys(
		response: Response<Body>,
		mut expected_keys: Vec<[u8; 32]>,
	) -> Result<()> {
		assert_eq!(response.status(), StatusCode::OK);
		assert_eq!(response.headers()["Content-Type"], "application/json");
		let body = response.into_body().collect().await?.to_bytes();
		let jwks: JwkSet =
			serde_json::from_slice(&body).wrap_err("failed to deserialize response")?;
		let mut ed25519_keys: Vec<[u8; 32]> = jwks
			.keys
			.into_iter()
			.map(|jwk| {
				let jose_jwk::Key::Okp(ref key) = jwk.key else {
					panic!("did not encounter okp key group");
				};
				assert_eq!(key.crv, OkpCurves::Ed25519);
				assert!(key.d.is_none(), "private keys should not be stored");
				let key: [u8; 32] =
					key.x.as_ref().try_into().expect("wrong key length");
				key
			})
			.collect();

		ed25519_keys.sort();
		expected_keys.sort();
		assert_eq!(ed25519_keys, expected_keys);

		Ok(())
	}

	/// Puts `num` as last byte of pubkey, everything else zero.
	fn key_from_number(num: u8) -> [u8; 32] {
		let mut expected_key = [0; 32];
		*expected_key.last_mut().unwrap() = num;
		expected_key
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_read_db(db_pool: SqlitePool) -> Result<()> {
		let router = test_router(db_pool).await?;
		let req = Request::builder()
			.method("GET")
			.uri(format!("/users/{}/did.json", Uuid::from_u128(1)))
			.body(axum::body::Body::empty())
			.unwrap();
		let response = router.oneshot(req).await?;

		check_response_keys(response, vec![key_from_number(1)]).await
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_read_nonexistent_user(db_pool: SqlitePool) -> Result<()> {
		let router = test_router(db_pool).await?;
		let req = Request::builder()
			.method("GET")
			.uri(format!("/users/{}/did.json", Uuid::nil()))
			.body(axum::body::Body::empty())
			.unwrap();
		let response = router.oneshot(req).await?;

		assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

		Ok(())
	}
}
