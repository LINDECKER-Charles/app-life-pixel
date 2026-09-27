//! In-memory adapters of the ports, for tests: feature `testing`.

mod clock;
mod events;
mod ids;
mod library_store;
mod owner_library;

pub use clock::FixedClock;
pub use events::RecordingEvents;
pub use ids::SequentialIds;
pub use library_store::InMemoryLibraryStore;
