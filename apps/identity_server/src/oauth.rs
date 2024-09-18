//! Routes for handling oauth with Google.

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::post, Form, Router};
use axum_extra::extract::cookie::CookieJar;
use color_eyre::eyre::{eyre, OptionExt, WrapErr as _};
use jsonwebtoken::DecodingKey;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::google_jwks_provider::JwksProvider;

#[derive(Debug, Clone)]
struct RouterState {
	google_jwt_validation: jsonwebtoken::Validation,
	google_jwks_provider: Arc<JwksProvider>,
}

#[derive(Debug)]
pub struct OAuthConfig {
	pub google_client_id: String,
	/// ArcSwap is used, so that another task can continuously refresh the keys.
	pub google_jwks_provider: JwksProvider,
}

impl OAuthConfig {
	pub async fn build(self) -> color_eyre::Result<Router> {
		let google_jwt_validation = {
			let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
			v.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
			v.set_audience(&[self.google_client_id]);
			v
		};
		Ok(Router::new()
			.route("/google", post(google))
			.with_state(RouterState {
				google_jwt_validation,
				google_jwks_provider: Arc::new(self.google_jwks_provider),
			}))
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleIdForm {
	credential: String,
	g_csrf_token: String,
}

#[derive(thiserror::Error, Debug)]
enum GoogleErr {
	#[error(transparent)]
	Internal(#[from] color_eyre::eyre::Report),
}

impl IntoResponse for GoogleErr {
	fn into_response(self) -> axum::response::Response {
		error!("{self:?}");
		match self {
			Self::Internal(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
			}
		}
	}
}

/// See <https://developers.google.com/identity/gsi/web/reference/html-reference>
#[derive(Debug, Serialize, Deserialize)]
struct GoogleIdTokenClaims {
	/// Unique ID of user's google account.
	sub: String,
	name: String,
	email: String,
}

#[tracing::instrument(skip_all)]
#[axum_macros::debug_handler]
async fn google(
	State(state): State<RouterState>,
	jar: CookieJar,
	Form(form): Form<GoogleIdForm>,
) -> Result<(), GoogleErr> {
	// Check for CSRF
	let cookie = jar
		.get("g_csrf_token")
		.ok_or_eyre("missing the double-submit csrf cookie")?;
	if form.g_csrf_token != cookie.value() {
		return Err(eyre!("double-submit csrf cookie mismatched!").into());
	}

	let google_keys = state
		.google_jwks_provider
		.get()
		.await
		.wrap_err("failed to get google's public keys")?;
	debug!(?form, "received form");
	let token = &form.credential;
	let header =
		jsonwebtoken::decode_header(token).wrap_err("could not decode JWT header")?;

	// TODO: Start caching the decoding keys in a HashMap.
	let decoding_key = {
		let Some(ref token_key_id) = header.kid else {
			return Err(eyre!("expected a `kid` field in the jwt header").into());
		};
		let google_key = google_keys
			.jwks()
			.keys
			.iter()
			.find(|jwk| jwk.common.key_id.as_ref() == Some(token_key_id))
			.ok_or_eyre(
				"the provided credential's key did not match google's reported keys",
			)?;

		DecodingKey::from_jwk(google_key)
			.wrap_err("failed to create decoding key from jwk")?
	};

	let decoded_jwt = jsonwebtoken::decode::<GoogleIdTokenClaims>(
		&form.credential,
		&decoding_key,
		&state.google_jwt_validation,
	)
	.wrap_err("failed to validate jwt")?;
	info!(claims = ?decoded_jwt.claims, "Got ID Token claims");
	// TODO: Do something with the user info that we got
	Ok(())
}
