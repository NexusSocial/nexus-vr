pub mod jwk;
pub mod v1;

mod uuid;

use axum::routing::get;
use color_eyre::eyre::Context as _;
use sqlx::sqlite::SqlitePool;
use tower_http::trace::TraceLayer;

pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// A [`SqlitePool`] that has already been migrated.
#[derive(Debug, Clone)]
pub struct MigratedDbPool(SqlitePool);

impl MigratedDbPool {
	pub async fn new(pool: SqlitePool) -> color_eyre::Result<Self> {
		MIGRATOR
			.run(&pool)
			.await
			.wrap_err("failed to run migrations")?;

		Ok(Self(pool))
	}
}

#[derive(Debug)]
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
