//! Support requests (H9): Help → Contact of docs/admin-console.md. A person signed in sends a
//! category, a message, the context the app attaches, and an optional screenshot; they read their
//! requests and the team's replies, and reply in turn.
//!
//! [`Support`] holds the use cases, over two ports of their own — [`SupportStore`] and
//! [`ScreenshotStore`] — and the clock, ids and product events of [`crate::ports`]. A screenshot
//! is checked, decoded under the limits and re-encoded as PNG on a blocking thread
//! ([`Screenshot`]), a few at a time. With the `testing` feature, [`memory`] holds in-memory
//! adapters of the ports and sample uploads.
//!
//! [`SupportStore`]: ports::SupportStore
//! [`ScreenshotStore`]: ports::ScreenshotStore

mod cases;
mod error;
pub mod ports;
mod screenshot;
mod values;

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use tokio::sync::Semaphore;

pub use cases::SupportSubmission;
pub use error::SupportError;
pub use screenshot::{Screenshot, ScreenshotRejection};
pub use values::{
    Author, CONTEXT_FIELD_MAX_CHARS, Category, MESSAGE_MIN_CHARS, MessageBody, SCREEN_MAX_CHARS,
    SupportContext, SupportRequestId, SupportStatus,
};

use self::ports::{ScreenshotStore, SupportStore};
use crate::ports::{Clock, EventSink, IdGenerator};

/// How many screenshots are decoded at once, whatever the number of requests: each may take
/// tens of megabytes.
pub const SCREENSHOT_DECODES_AT_ONCE: usize = 2;
/// The path, after `LP_PUBLIC_URL`, of a request's thread in the app: what the team's reply
/// emails link to, followed by the request's id.
pub const THREAD_PATH: &str = "/support/";

/// The stores the support use cases keep requests and screenshots in.
#[derive(Clone)]
pub struct SupportStores {
    /// Where requests and their messages are kept.
    pub requests: Arc<dyn SupportStore>,
    /// Where screenshots are kept.
    pub screenshots: Arc<dyn ScreenshotStore>,
}

/// The adapters the support use cases work through.
#[derive(Clone)]
pub struct SupportPorts {
    /// Where requests and screenshots are kept.
    pub stores: SupportStores,
    /// The time of requests and messages.
    pub clock: Arc<dyn Clock>,
    /// New request and message ids.
    pub ids: Arc<dyn IdGenerator>,
    /// Where product events go.
    pub events: Arc<dyn EventSink>,
}

/// The support use cases.
#[derive(Clone)]
pub struct Support {
    ports: SupportPorts,
    decodes: Arc<Semaphore>,
}

impl Support {
    /// The use cases over `ports`.
    #[must_use]
    pub fn new(ports: SupportPorts) -> Self {
        Self {
            ports,
            decodes: Arc::new(Semaphore::new(SCREENSHOT_DECODES_AT_ONCE)),
        }
    }

    fn requests(&self) -> &dyn SupportStore {
        self.ports.stores.requests.as_ref()
    }

    fn screenshots(&self) -> &dyn ScreenshotStore {
        self.ports.stores.screenshots.as_ref()
    }
}
