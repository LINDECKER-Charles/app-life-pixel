//! Personal access tokens on Postgres (A3): the `access_tokens` table, by the SHA-256 of each
//! token's secret, as `service::tokens::ports` describes it.

mod store;

use std::sync::Arc;

use life_pixel_service::tokens::ports::TokenStore;
use sqlx::PgPool;

pub use store::PostgresTokenStore;

/// The token store over the migrated database of `pool`.
#[must_use]
pub fn hosted_store(pool: &PgPool) -> Arc<dyn TokenStore> {
    Arc::new(PostgresTokenStore::new(pool.clone()))
}
