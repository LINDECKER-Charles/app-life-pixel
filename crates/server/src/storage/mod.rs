//! The hosted library's storage: the index in Postgres, the documents in object storage, the
//! sweeper of the objects no row references, and their metrics; and the accounts' stores. One line
//! per module.

mod accounts;
mod hosted;
mod keys;
pub mod metrics;
mod objects;
mod sweeper;
mod upkeep;

pub use accounts::{PostgresAccountStore, PostgresEmailTokenStore, PostgresSessionStore};
pub use hosted::HostedLibraryStore;
pub use keys::{DOCUMENTS_PREFIX, animation_prefix, new_document_key};
pub use objects::{ObjectStoreSetupError, object_store};
pub use sweeper::{ORPHAN_MIN_AGE, SWEEP_BATCH_KEYS, SWEEP_PERIOD, SweepError, Sweeper};
pub use upkeep::spawn_upkeep;
