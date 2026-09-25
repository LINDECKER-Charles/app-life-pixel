//! Readiness: whether the server's dependencies answer, for `/healthz`.

use async_trait::async_trait;

/// Whether the database answers: `database::DatabaseReadiness` runs a query through the pool;
/// the router tests put a stand-in in its place.
#[async_trait]
pub trait Readiness: Send + Sync {
    /// Whether the dependency answers now.
    async fn is_ready(&self) -> bool;
}
