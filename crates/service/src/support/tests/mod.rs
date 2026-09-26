//! The support use cases over the in-memory adapters.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

mod creation;
mod screenshots;
mod thread;

use std::sync::Arc;

use bytes::Bytes;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

use super::memory::{InMemoryScreenshotStore, InMemorySupportStore};
use super::ports::SupportThread;
use super::{Support, SupportPorts, SupportStores, SupportSubmission};
use crate::ids::AccountId;
use crate::memory::{FixedClock, RecordingEvents, SequentialIds};

/// When the tests start.
const START: OffsetDateTime = datetime!(2026-09-01 12:00 UTC);
/// The context the app attaches.
const CONTEXT: &str =
    r#"{"appVersion":"0.1.0","platform":"web","language":"fr","screen":"/editor/:animationId"}"#;

/// The use cases and the adapters they work through.
struct Harness {
    support: Support,
    requests: Arc<InMemorySupportStore>,
    screenshots: Arc<InMemoryScreenshotStore>,
    clock: Arc<FixedClock>,
    events: Arc<RecordingEvents>,
}

impl Harness {
    fn new() -> Self {
        let requests = Arc::new(InMemorySupportStore::new());
        let screenshots = Arc::new(InMemoryScreenshotStore::new());
        let clock = Arc::new(FixedClock::new(START));
        let events = Arc::new(RecordingEvents::new());
        let ports = SupportPorts {
            stores: SupportStores {
                requests: requests.clone(),
                screenshots: screenshots.clone(),
            },
            clock: clock.clone(),
            ids: Arc::new(SequentialIds::new()),
            events: events.clone(),
        };
        Self {
            support: Support::new(ports),
            requests,
            screenshots,
            clock,
            events,
        }
    }

    /// Sends `message` as a bug for `account`, without a screenshot.
    async fn send(&self, account: AccountId, message: &str) -> SupportThread {
        self.support
            .create(account, submission(message, None))
            .await
            .unwrap()
    }
}

/// A bug report saying `message`, with the app's context and `screenshot`.
fn submission(message: &str, screenshot: Option<Vec<u8>>) -> SupportSubmission {
    SupportSubmission {
        category: "bug".into(),
        message: message.into(),
        context: CONTEXT.as_bytes().to_vec(),
        screenshot: screenshot.map(Bytes::from),
    }
}

/// The account numbered `n`.
fn account(n: u128) -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(n))
}
