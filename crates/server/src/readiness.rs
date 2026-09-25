//! Readiness: whether the server's dependencies answer, for `/healthz`.

use async_trait::async_trait;

/// Whether the database answers. H3 checks that its address accepts a connection; H4 puts the
/// pool's check in its place.
#[async_trait]
pub trait Readiness: Send + Sync {
    /// Whether the dependency answers now.
    async fn is_ready(&self) -> bool;
}
