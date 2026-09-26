//! Where support requests and their messages are kept.

use async_trait::async_trait;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};
use crate::support::{Author, Category, SupportContext, SupportRequestId, SupportStatus};

/// A request as its author sees it, without its messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportRequest {
    /// Its id.
    pub id: SupportRequestId,
    /// What it is about.
    pub category: Category,
    /// Where it stands.
    pub status: SupportStatus,
    /// Whether a screenshot came with it.
    pub has_screenshot: bool,
    /// When it was sent.
    pub created_at: OffsetDateTime,
    /// When it last changed.
    pub updated_at: OffsetDateTime,
}

/// A message of a request, as its author sees it: never an internal note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportMessage {
    /// Its id.
    pub id: Uuid,
    /// Who wrote it.
    pub author: Author,
    /// Its text.
    pub body: String,
    /// When it was written.
    pub created_at: OffsetDateTime,
}

/// A request and its messages, oldest first: the first is the request's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportThread {
    /// The request.
    pub request: SupportRequest,
    /// Its messages, internal notes left out.
    pub messages: Vec<SupportMessage>,
}

/// A request to store, with its first message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewSupportRequest {
    /// The account that sends it.
    pub account: AccountId,
    /// The request, status `new`.
    pub request: SupportRequest,
    /// What the app attached.
    pub context: SupportContext,
    /// Where its screenshot is stored, if any.
    pub screenshot_key: Option<String>,
    /// Its first message, by the user.
    pub message: SupportMessage,
}

/// A reply of the user to one of their requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUserMessage {
    /// The account replying.
    pub account: AccountId,
    /// The request replied to.
    pub request: SupportRequestId,
    /// The reply, by the user.
    pub message: SupportMessage,
}

/// What became of a reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserReply {
    /// Stored; the request now has `status`.
    Added {
        /// The request's status after the reply.
        status: SupportStatus,
    },
    /// The account has no such request.
    NotFound,
    /// The request's status takes no reply.
    Refused,
}

/// A store failure; the detail is for the logs, and never holds an address or a message.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("support storage unavailable: {0}")]
pub struct SupportStoreError(pub String);

/// Keeps support requests, their messages and their context.
#[async_trait]
pub trait SupportStore: Send + Sync {
    /// Stores `new`, its request and its first message together.
    async fn create(&self, new: NewSupportRequest) -> Result<(), SupportStoreError>;

    /// The requests of `account`, from the most recently updated.
    async fn list(
        &self,
        account: AccountId,
        page: PageRequest,
    ) -> Result<Page<SupportRequest>, SupportStoreError>;

    /// The request `id` of `account` with its messages, internal notes left out.
    async fn thread(
        &self,
        account: AccountId,
        id: SupportRequestId,
    ) -> Result<Option<SupportThread>, SupportStoreError>;

    /// Adds `reply` to its request, which moves to the status
    /// [`SupportStatus::after_user_reply`] gives, and whose `updated_at` becomes the reply's
    /// time; it refuses the reply when that gives none. The status is read and changed under
    /// one lock, so that a change by the team in between is never lost.
    async fn add_user_message(&self, reply: NewUserMessage)
    -> Result<UserReply, SupportStoreError>;
}
