//! The library use cases: projects, animations and their documents, under the storage quota.

mod account;
mod animations;
mod copies;
mod documents;
mod error;
mod parsing;
mod projects;

use std::sync::Arc;

pub use account::Usage;
pub use error::LibraryError;

use crate::owner::Owner;
use crate::plans::Plans;
use crate::ports::events::ProductEvent;
use crate::ports::library_store::StoreError;
use crate::ports::{Clock, EventSink, IdGenerator, LibraryStore};

/// An animation was created, imported or duplicated.
const ANIMATION_CREATED: &str = "animation_created";
/// A document was saved; property [`SIZE`].
const DOCUMENT_SAVED: &str = "document_saved";
/// A create, a save or a copy was refused by the storage quota.
const QUOTA_REJECTED: &str = "quota_rejected";
/// The size class of a saved document.
const SIZE: &str = "size";

/// The adapters the library works through.
#[derive(Clone)]
pub struct LibraryPorts {
    /// Where projects and animations are kept.
    pub store: Arc<dyn LibraryStore>,
    /// The time of creations and changes.
    pub clock: Arc<dyn Clock>,
    /// New project and animation ids.
    pub ids: Arc<dyn IdGenerator>,
    /// Where product events go.
    pub events: Arc<dyn EventSink>,
}

/// The library use cases. An [`Owner::Account`] gets the free plan's storage quota,
/// [`Owner::Local`] none; only accounts record product events.
#[derive(Clone)]
pub struct Library {
    ports: LibraryPorts,
    plans: Plans,
}

impl Library {
    /// The library over `ports`, with the plan values of configuration.
    #[must_use]
    pub fn new(ports: LibraryPorts, plans: Plans) -> Self {
        Self { ports, plans }
    }

    fn store(&self) -> &dyn LibraryStore {
        self.ports.store.as_ref()
    }

    fn quota(&self, owner: &Owner) -> Option<u64> {
        owner.account().map(|_| self.plans.free_storage_bytes)
    }

    /// Records the event `name` for an account; the local library sends nothing.
    fn record(&self, owner: &Owner, name: &'static str) {
        if let Some(event) = account_event(owner, name) {
            self.ports.events.record(event);
        }
    }

    /// Records a save of `document_bytes` for an account, with its size class.
    fn record_saved(&self, owner: &Owner, document_bytes: u64) {
        let Some(mut event) = account_event(owner, DOCUMENT_SAVED) else {
            return;
        };
        let size = ProductEvent::size_class(document_bytes).to_owned();
        event.properties.push((SIZE, size));
        self.ports.events.record(event);
    }

    /// The use-case error of `error`, recording a quota refusal.
    fn refused(&self, owner: &Owner, error: StoreError) -> LibraryError {
        if matches!(error, StoreError::QuotaExceeded { .. }) {
            self.record(owner, QUOTA_REJECTED);
        }
        error.into()
    }
}

/// The event `name` about the owner's account, without properties; `None` for the local library.
fn account_event(owner: &Owner, name: &'static str) -> Option<ProductEvent> {
    owner.account().map(|account| ProductEvent {
        name,
        account: Some(account),
        properties: Vec::new(),
    })
}
