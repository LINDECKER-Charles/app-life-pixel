//! The support use cases: sending a request, and reading and answering one's own.

mod creation;
mod thread;

pub use creation::SupportSubmission;

use time::OffsetDateTime;
use uuid::Uuid;

use super::ports::SupportMessage;
use super::{Author, MessageBody, Support};

impl Support {
    /// A new message of the user, written now.
    fn user_message(&self, body: MessageBody, at: OffsetDateTime) -> SupportMessage {
        SupportMessage {
            id: self.new_id(),
            author: Author::User,
            body: body.into_string(),
            created_at: at,
        }
    }

    fn new_id(&self) -> Uuid {
        self.ports.ids.new_id()
    }
}
