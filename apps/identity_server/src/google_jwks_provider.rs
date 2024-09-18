use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use axum::async_trait;
use color_eyre::{eyre::WrapErr as _, Result, Section};
use jsonwebtoken::jwk::JwkSet;
use reqwest::Url;
use tracing::info;

/// Retrieves the latest JWKs for an external service.
///
/// Example: This can be used to get the JWKs from google, located at
/// <https://www.googleapis.com/oauth2/v3/certs>
///
/// This provider exists to support mocking of the external interface, for the purposes
/// of testing.
#[derive(Debug)]
pub struct JwksProvider {
	#[cfg(not(test))]
	provider: HttpProvider,
	#[cfg(test)]
	provider: Box<dyn JwksProviderT>,
}

impl JwksProvider {
	pub fn google(client: reqwest::Client) -> Self {
		Self {
			#[cfg(not(test))]
			provider: HttpProvider::google(client),
			#[cfg(test)]
			provider: Box::new(HttpProvider::google(client)),
		}
	}
	pub async fn get(&self) -> Result<Arc<CachedJwks>> {
		self.provider.get().await
	}
}

#[async_trait]
trait JwksProviderT: std::fmt::Debug + Send + Sync + 'static {
	/// Gets the latest JWKS for google.
	async fn get(&self) -> Result<Arc<CachedJwks>>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct CachedJwks {
	jwks: JwkSet,
	expires_at: std::time::Instant,
}

impl CachedJwks {
	/// Creates an empty set of JWKs, which is already expired.
	fn new_expired() -> Self {
		let now = std::time::Instant::now();
		let expires_at = now.checked_sub(Duration::from_secs(1)).unwrap_or(now);
		Self {
			jwks: JwkSet { keys: vec![] },
			expires_at,
		}
	}

	pub fn jwks(&self) -> &JwkSet {
		&self.jwks
	}

	fn is_expired(&self) -> bool {
		self.expires_at <= std::time::Instant::now()
	}
}

/// Uses http to retrieve the JWKs.
#[derive(Debug)]
struct HttpProvider {
	url: Url,
	client: reqwest::Client,
	cached_jwks: ArcSwap<CachedJwks>,
}

impl HttpProvider {
	/// Creates a provider that requests the JWKS over HTTP from google's url.
	pub fn google(client: reqwest::Client) -> Self {
		// Creates immediately expired empty keyset
		Self {
			client,
			url: "https://www.googleapis.com/oauth2/v3/certs"
				.try_into()
				.unwrap(),
			cached_jwks: ArcSwap::new(Arc::new(CachedJwks::new_expired())),
		}
	}
}

#[async_trait]
impl JwksProviderT for HttpProvider {
	/// Usually this is instantly ready with the JWKS, but if the cached value doesn't
	/// exist
	/// or is out of date, it will await on the new value.
	async fn get(&self) -> Result<Arc<CachedJwks>> {
		let cached_jwks = self.cached_jwks.load();
		if !cached_jwks.is_expired() {
			return Ok(cached_jwks.to_owned());
		}
		let response = self
			.client
			.get(self.url.clone())
			.send()
			.await
			.wrap_err("failed to initiate get request for certs")
			.with_note(|| format!("url was {}", self.url))?;
		let expires_at = {
			if let Some(duration) =
				header_parsing::time_until_max_age(response.headers())
			{
				std::time::Instant::now() + duration
			} else {
				std::time::Instant::now()
			}
		};
		let serialized_keys = response
			.bytes()
			.await
			.wrap_err("failed to get response body")?;
		let jwks: JwkSet = serde_json::from_slice(&serialized_keys)
			.wrap_err("unexpected response, expected a JWKS")?;
		let cached_jwks = Arc::new(CachedJwks { jwks, expires_at });
		self.cached_jwks.store(Arc::clone(&cached_jwks));
		info!("cached JWKs: {cached_jwks:?}");
		Ok(cached_jwks)
	}
}

/// Always provides the same JWKs.
#[derive(Debug, Clone)]
struct StaticProvider(Arc<CachedJwks>);

#[async_trait]
impl JwksProviderT for StaticProvider {
	async fn get(&self) -> Result<Arc<CachedJwks>> {
		Ok(Arc::clone(&self.0))
	}
}
