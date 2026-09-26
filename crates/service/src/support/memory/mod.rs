//! In-memory adapters of the support ports, and sample uploads, for tests: feature `testing`.

mod samples;
mod screenshot_store;
mod support_store;

pub use samples::{jpeg_screenshot, png_claiming_side, png_screenshot_with_text};
pub use screenshot_store::InMemoryScreenshotStore;
pub use support_store::InMemorySupportStore;

use std::sync::Arc;

use super::SupportStores;

/// Support stores in memory, for the tests that need them without looking inside.
#[must_use]
pub fn in_memory_stores() -> SupportStores {
    SupportStores {
        requests: Arc::new(InMemorySupportStore::new()),
        screenshots: Arc::new(InMemoryScreenshotStore::new()),
    }
}
