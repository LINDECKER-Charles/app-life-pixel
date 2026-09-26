//! What the support use cases need from outside, besides the clock, ids and product events of
//! [`crate::ports`]: the server implements them on Postgres and object storage.

mod screenshot_store;
mod support_store;

pub use screenshot_store::ScreenshotStore;
pub use support_store::{
    NewSupportRequest, NewUserMessage, SupportMessage, SupportRequest, SupportStore,
    SupportStoreError, SupportThread, UserReply,
};
