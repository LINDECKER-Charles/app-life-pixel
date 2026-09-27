//! A library over the in-memory adapters, shared by the use-case tests.

#![allow(dead_code)] // Each test file uses its own share of the helpers.
#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

pub mod unavailable;

use std::sync::Arc;

use bytes::Bytes;
use life_pixel_core::limits::{CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, NAME_MAX_CHARS};
use life_pixel_core::{Name, NewAnimation};
use life_pixel_service::library::{Library, LibraryError, LibraryPorts};
use life_pixel_service::memory::{
    FixedClock, InMemoryLibraryStore, RecordingEvents, SequentialIds,
};
use life_pixel_service::ports::ProductEvent;
use life_pixel_service::ports::library_store::{AnimationRecord, ProjectRecord};
use life_pixel_service::{AccountId, Coded, Owner, Plans, ProjectId};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

/// The free plan's quota in these tests: roomy enough for a few small documents.
pub const FREE_STORAGE_BYTES: u64 = 100_000;
/// When the tests start.
pub const START: OffsetDateTime = datetime!(2026-09-01 12:00 UTC);

/// An account of the hosted service.
pub const ACCOUNT: Owner = Owner::Account(AccountId::from_uuid(Uuid::from_u128(0xacc0)));
/// Another account.
pub const OTHER_ACCOUNT: Owner = Owner::Account(AccountId::from_uuid(Uuid::from_u128(0xacc1)));

/// A library and the adapters it works through.
pub struct Harness {
    pub library: Library,
    pub clock: Arc<FixedClock>,
    pub events: Arc<RecordingEvents>,
}

impl Harness {
    /// A library whose free plan allows `free_storage_bytes`.
    pub fn with_quota(free_storage_bytes: u64) -> Self {
        Self::over(Arc::new(InMemoryLibraryStore::new()), free_storage_bytes)
    }

    /// A library over `store`, whose free plan allows `free_storage_bytes`.
    pub fn over(store: Arc<dyn life_pixel_service::ports::LibraryStore>, quota: u64) -> Self {
        let clock = Arc::new(FixedClock::new(START));
        let events = Arc::new(RecordingEvents::new());
        let ports = LibraryPorts {
            store,
            clock: clock.clone(),
            ids: Arc::new(SequentialIds::new()),
            events: events.clone(),
        };
        let plans = Plans {
            free_storage_bytes: quota,
            free_mcp_calls_per_day: 0,
        };
        let library = Library::new(ports, plans);
        Self {
            library,
            clock,
            events,
        }
    }

    /// The names of the events recorded so far.
    pub fn event_names(&self) -> Vec<&'static str> {
        self.events
            .events()
            .iter()
            .map(|event| event.name)
            .collect()
    }

    /// The events recorded so far.
    pub fn events(&self) -> Vec<ProductEvent> {
        self.events.events()
    }

    /// A new project of `owner`.
    pub async fn project(&self, owner: &Owner, name: &str) -> ProjectRecord {
        self.library.create_project(owner, name).await.unwrap()
    }

    /// A new animation of [`ACCOUNT`] in `project`, imported from the sample document `title`.
    pub async fn animation(&self, project: ProjectId, title: &str) -> AnimationRecord {
        let (_, document) = sample(title);
        let imported = self.library.import_animation(&ACCOUNT, project, document);
        imported.await.unwrap()
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::with_quota(FREE_STORAGE_BYTES)
    }
}

/// The sample document titled `title`.
pub fn sample(title: &str) -> (life_pixel_service::ports::AnimationMeta, Bytes) {
    life_pixel_service::testing::sample_document(title)
}

/// The spec of a blank `width` × `height` animation titled `title`.
pub fn spec(title: &str, width: u16, height: u16) -> NewAnimation {
    NewAnimation {
        title: Name::new(title).unwrap(),
        width,
        height,
        layer_name: Name::new("Layer 1").unwrap(),
        frame_duration_ms: None,
        palette: None,
    }
}

/// Asserts that `error` has the code `code` and the parameters `params`.
pub fn assert_coded(error: &LibraryError, code: &str, params: Value) {
    assert_eq!(error.code(), code, "{error:?}");
    let expected: Map<String, Value> = match params {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    assert_eq!(error.params(), expected, "{error:?}");
}

/// Asserts that `error` has the code `code` and no parameter.
pub fn assert_code(error: &LibraryError, code: &str) {
    assert_coded(error, code, json!({}));
}

/// The parameters of `document.name`.
pub fn name_params() -> Value {
    json!({ "max": NAME_MAX_CHARS })
}

/// The parameters of `document.canvas_size`.
pub fn canvas_params() -> Value {
    json!({ "min": CANVAS_MIN_SIDE, "max": CANVAS_MAX_SIDE })
}
