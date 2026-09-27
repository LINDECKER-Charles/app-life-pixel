//! What the use cases need from outside: time, ids, product events and storage. The server, the
//! CLI and the desktop app provide the adapters.

pub mod clock;
pub mod events;
pub mod ids;
pub mod library_store;

pub use clock::{Clock, SystemClock};
pub use events::{EventSink, ProductEvent};
pub use ids::{IdGenerator, UuidV7Ids};
pub use library_store::{
    AnimationFilter, AnimationMeta, AnimationRecord, DocumentWrite, LibraryStore,
    NewAnimationRecord, ProjectRecord, StoreError,
};
