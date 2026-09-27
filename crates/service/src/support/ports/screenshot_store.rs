//! Where the screenshots of support requests are kept.

use async_trait::async_trait;
use bytes::Bytes;

use super::SupportStoreError;
use crate::support::SupportRequestId;

/// Keeps one screenshot per request, already re-encoded as PNG. Nothing serves it back to the
/// person who sent it; only the team reads it.
#[async_trait]
pub trait ScreenshotStore: Send + Sync {
    /// Stores `png` as the screenshot of `request`; returns its key.
    async fn put(&self, request: SupportRequestId, png: Bytes)
    -> Result<String, SupportStoreError>;

    /// The screenshot at `key`.
    async fn read(&self, key: &str) -> Result<Bytes, SupportStoreError>;

    /// Deletes the screenshot at `key`.
    async fn delete(&self, key: &str) -> Result<(), SupportStoreError>;
}
