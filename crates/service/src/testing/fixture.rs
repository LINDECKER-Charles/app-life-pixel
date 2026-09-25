//! What a contract case runs on.

use std::any::Any;
use std::sync::Arc;

use uuid::Uuid;

use crate::ids::AccountId;
use crate::owner::Owner;
use crate::ports::LibraryStore;

/// An adapter under test, the two owners the cases act for, and whatever must live as long as
/// the adapter — a temporary folder, a test database.
pub struct StoreFixture {
    store: Arc<dyn LibraryStore>,
    owner: Owner,
    other_owner: Owner,
    /// Dropped with the fixture, never read.
    _guard: Option<Box<dyn Any + Send + Sync>>,
}

impl StoreFixture {
    /// `store`, for two new accounts.
    pub fn new(store: impl LibraryStore + 'static) -> Self {
        Self {
            store: Arc::new(store),
            owner: new_account(),
            other_owner: new_account(),
            _guard: None,
        }
    }

    /// The fixture acting for `owner`, and checking that `other_owner` sees none of its data —
    /// `Owner::Local` and an account for the local library.
    #[must_use]
    pub fn with_owners(self, owner: Owner, other_owner: Owner) -> Self {
        Self {
            owner,
            other_owner,
            ..self
        }
    }

    /// The fixture keeping `guard` alive until the case ends.
    #[must_use]
    pub fn with_guard(self, guard: impl Any + Send + Sync) -> Self {
        Self {
            _guard: Some(Box::new(guard)),
            ..self
        }
    }

    /// The adapter under test.
    #[must_use]
    pub fn store(&self) -> &Arc<dyn LibraryStore> {
        &self.store
    }

    /// The owner the cases act for.
    #[must_use]
    pub fn owner(&self) -> Owner {
        self.owner
    }

    /// An owner that must see none of [`owner`](Self::owner)'s data.
    #[must_use]
    pub fn other_owner(&self) -> Owner {
        self.other_owner
    }
}

impl<S: LibraryStore + 'static> From<S> for StoreFixture {
    fn from(store: S) -> Self {
        Self::new(store)
    }
}

fn new_account() -> Owner {
    Owner::Account(AccountId::from_uuid(Uuid::now_v7()))
}
