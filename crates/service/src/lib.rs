//! The Life Pixel use cases and the ports they need.
//!
//! The crate knows nothing of HTTP, Tauri or MCP: the server, the CLI and the desktop app call
//! [`library::Library`] with their own adapters of the [`ports`]. Its async functions run on
//! tokio; parsing and validating a document through `core` goes to `spawn_blocking`.
//!
//! With the `testing` feature, [`memory`] holds in-memory adapters and [`testing`] the contract
//! suites every adapter runs from its own crate's tests.

pub mod accounts;
pub mod animation;
pub mod error;
pub mod events;
pub mod ids;
pub mod library;
pub mod local;
pub mod owner;
pub mod paging;
pub mod plans;
pub mod ports;
pub mod quota;

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(feature = "testing")]
pub mod testing;

pub use error::{Coded, CodedError};
pub use ids::{AccountId, AnimationId, ProjectId};
pub use owner::Owner;
pub use paging::{Cursor, Page, PageRequest};
pub use plans::Plans;
