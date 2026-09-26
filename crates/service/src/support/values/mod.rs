//! The values of a support request, each checked where it is made.

mod author;
mod category;
mod context;
mod message_body;
mod request_id;
mod status;

pub use author::Author;
pub use category::Category;
pub use context::{CONTEXT_FIELD_MAX_CHARS, SCREEN_MAX_CHARS, SupportContext};
pub use message_body::{MESSAGE_MIN_CHARS, MessageBody};
pub use request_id::SupportRequestId;
pub use status::SupportStatus;
