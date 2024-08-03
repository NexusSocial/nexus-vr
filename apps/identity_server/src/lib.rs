pub mod v1;

use axum::routing::get;
use tower_http::trace::TraceLayer;

/// Main router of API
pub fn router() -> axum::Router<()> {
	axum::Router::new()
		.route("/", get(root))
		.nest("/api/v1", crate::v1::router())
		.layer(TraceLayer::new_for_http())
}

async fn root() -> &'static str {
	"uwu hewwo this api is under constwuction"
}
