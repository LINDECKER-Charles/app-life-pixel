//! The support requests' stores (H9): the requests and their messages in Postgres, their
//! screenshots in object storage under `support/`.

mod request_store;
mod rows;
mod screenshot_store;

pub use request_store::PostgresSupportStore;
pub use screenshot_store::ObjectScreenshotStore;
