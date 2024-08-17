pub mod jwk;
pub mod v1;

mod uuid;

use axum::routing::get;
use color_eyre::eyre::Context as _;
use tower_http::trace::TraceLayer;

/// Main router of API
#[derive(Debug, Default)]
pub struct RouterConfig {
	pub v1: crate::v1::RouterConfig,
}

impl RouterConfig {
	pub async fn build(self) -> color_eyre::Result<axum::Router<()>> {
		let v1 = self
			.v1
			.build()
			.await
			.wrap_err("failed to build v1 router")?;
		Ok(axum::Router::new()
			.route("/", get(root))
			.nest("/api/v1", v1)
			.layer(TraceLayer::new_for_http()))
	}
}

async fn root() -> &'static str {
	"uwu hewwo this api is under constwuction"
}
