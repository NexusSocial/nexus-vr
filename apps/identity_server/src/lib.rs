mod uuid;
pub mod v1;

use axum::routing::get;
use tower_http::trace::TraceLayer;

/// Main router of API
pub fn router() -> axum::Router<()> {
	let v1_router = crate::v1::RouterConfig {
		..Default::default()
	}
	.build();
	axum::Router::new()
		.route("/", get(root))
		.nest("/api/v1", v1_router)
		.layer(TraceLayer::new_for_http())
}

async fn root() -> &'static str {
	"uwu hewwo this api is under constwuction"
}
