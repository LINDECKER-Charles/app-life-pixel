//! Where the team reads support requests and answers them.

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use super::AdminStoreError;
use crate::admin::{Assignee, AuditDraft};
use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};
use crate::support::ports::SupportRequest;
use crate::support::{Author, Category, SupportContext, SupportRequestId, SupportStatus};

/// Which requests the queue lists, from the most recently updated; every filter set must match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueueFilter {
    /// Only the requests of this status.
    pub status: Option<SupportStatus>,
    /// Only the requests of this category.
    pub category: Option<Category>,
    /// Only the requests assigned to this admin.
    pub assignee: Option<String>,
}

/// A request as the team sees it, without its messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminRequest {
    /// The request, as its author sees it.
    pub request: SupportRequest,
    /// The account that sent it.
    pub account: AccountId,
    /// That account's address.
    pub email: String,
    /// The admin it is assigned to.
    pub assigned_to: Option<String>,
    /// When the team first replied.
    pub first_response_at: Option<OffsetDateTime>,
    /// When it was resolved or closed, while it is.
    pub resolved_at: Option<OffsetDateTime>,
}

/// A message as the team sees it: internal notes included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminMessage {
    /// Its id.
    pub id: Uuid,
    /// Who wrote it.
    pub author: Author,
    /// The admin who wrote it, for the team's messages.
    pub admin_id: Option<String>,
    /// Its text.
    pub body: String,
    /// Whether it is a note the person never sees.
    pub is_internal: bool,
    /// When it was written.
    pub created_at: OffsetDateTime,
}

/// A request with its context and every message, oldest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminThread {
    /// The request.
    pub request: AdminRequest,
    /// What the app attached.
    pub context: SupportContext,
    /// Its messages, internal notes included.
    pub messages: Vec<AdminMessage>,
}

/// A change of a request's status or assignee, with its audit entry, in one transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestChange {
    /// The request.
    pub id: SupportRequestId,
    /// Its new status, if it changes.
    pub status: Option<SupportStatus>,
    /// Its new assignee — `Some(None)` for nobody —, if it changes.
    pub assigned_to: Option<Option<Assignee>>,
    /// The action, written with the state before and after.
    pub audit: AuditDraft,
}

/// A message of the team — a reply, or an internal note —, with its audit entry, in one
/// transaction. A reply also moves the request as [`super::RequestState::answered`] says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamMessage {
    /// The request.
    pub request: SupportRequestId,
    /// The message, by the team.
    pub message: AdminMessage,
    /// The action, written with the state before and after.
    pub audit: AuditDraft,
}

/// Whom a reply is emailed to: the account that sent the request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipient {
    /// Its address.
    pub email: String,
    /// Its language.
    pub language: String,
}

/// Where the team reads support requests and answers them.
#[async_trait]
pub trait AdminSupportStore: Send + Sync {
    /// A page of the requests `filter` keeps, from the most recently updated.
    async fn queue(
        &self,
        filter: QueueFilter,
        page: PageRequest,
    ) -> Result<Page<AdminRequest>, AdminStoreError>;

    /// The request `id` with its context and every message.
    async fn thread(&self, id: SupportRequestId) -> Result<Option<AdminThread>, AdminStoreError>;

    /// Where the screenshot of the request `id` is stored: `None` when there is no such request,
    /// `Some(None)` when it came without one.
    async fn screenshot_key(
        &self,
        id: SupportRequestId,
    ) -> Result<Option<Option<String>>, AdminStoreError>;

    /// Applies `change` with its audit entry; the request after it, or `None` when it does not
    /// exist, and nothing is written.
    async fn update(&self, change: RequestChange) -> Result<Option<AdminRequest>, AdminStoreError>;

    /// Adds `message` with its audit entry; whom to email a reply to, or `None` when the request
    /// does not exist, and nothing is written.
    async fn add_team_message(
        &self,
        message: TeamMessage,
    ) -> Result<Option<Recipient>, AdminStoreError>;
}
