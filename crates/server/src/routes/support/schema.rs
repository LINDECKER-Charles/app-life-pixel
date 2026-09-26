//! The support routes' bodies: requests, their messages and pages as the routes answer them,
//! and what the routes read.

use life_pixel_service::paging::{Cursor, PageRequest};
use life_pixel_service::support::ports::{SupportMessage, SupportRequest, SupportThread};
use life_pixel_service::support::{Author, Category, SupportStatus};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};

use crate::http::problem::Problem;

/// What a support request is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportCategory {
    /// Something does not work.
    Bug,
    /// Signing in, the address, the account.
    Account,
    /// Plans and payments.
    Billing,
    /// Data and privacy: access and erasure.
    DataProtection,
    /// Content or behaviour to report.
    Abuse,
    /// Anything else.
    Other,
}

impl From<Category> for SupportCategory {
    fn from(category: Category) -> Self {
        match category {
            Category::Bug => Self::Bug,
            Category::Account => Self::Account,
            Category::Billing => Self::Billing,
            Category::DataProtection => Self::DataProtection,
            Category::Abuse => Self::Abuse,
            Category::Other => Self::Other,
        }
    }
}

/// Where a support request stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportRequestStatus {
    /// Nobody from the team has looked at it yet.
    New,
    /// The team is on it.
    InProgress,
    /// The team answered, and waits for the person.
    WaitingForUser,
    /// Solved.
    Resolved,
    /// Over: it takes no more replies.
    Closed,
}

impl From<SupportStatus> for SupportRequestStatus {
    fn from(status: SupportStatus) -> Self {
        match status {
            SupportStatus::New => Self::New,
            SupportStatus::InProgress => Self::InProgress,
            SupportStatus::WaitingForUser => Self::WaitingForUser,
            SupportStatus::Resolved => Self::Resolved,
            SupportStatus::Closed => Self::Closed,
        }
    }
}

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportAuthor {
    /// The person who asked.
    User,
    /// The support team.
    Team,
}

/// A message of a request: never an internal note of the team.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportRequestMessage {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Who wrote it.
    pub author: SupportAuthor,
    /// Its text.
    pub body: String,
    /// When it was written.
    #[schema(format = DateTime)]
    pub created_at: String,
}

impl From<SupportMessage> for SupportRequestMessage {
    fn from(message: SupportMessage) -> Self {
        let author = match message.author {
            Author::User => SupportAuthor::User,
            Author::Team => SupportAuthor::Team,
        };
        Self {
            id: message.id.to_string(),
            author,
            body: message.body,
            created_at: timestamp(message.created_at),
        }
    }
}

/// A request of the account, without its messages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportRequestSummary {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// What it is about.
    pub category: SupportCategory,
    /// Where it stands.
    pub status: SupportRequestStatus,
    /// Whether a screenshot came with it; the screenshot itself is never served back.
    pub has_screenshot: bool,
    /// When it was sent.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it last changed.
    #[schema(format = DateTime)]
    pub updated_at: String,
}

impl From<SupportRequest> for SupportRequestSummary {
    fn from(request: SupportRequest) -> Self {
        Self {
            id: request.id.to_string(),
            category: request.category.into(),
            status: request.status.into(),
            has_screenshot: request.has_screenshot,
            created_at: timestamp(request.created_at),
            updated_at: timestamp(request.updated_at),
        }
    }
}

/// A request of the account and its messages, oldest first: the first is the request's own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportRequestThread {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// What it is about.
    pub category: SupportCategory,
    /// Where it stands.
    pub status: SupportRequestStatus,
    /// Whether a screenshot came with it; the screenshot itself is never served back.
    pub has_screenshot: bool,
    /// When it was sent.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it last changed.
    #[schema(format = DateTime)]
    pub updated_at: String,
    /// Its messages, the team's internal notes left out.
    pub messages: Vec<SupportRequestMessage>,
}

impl From<SupportThread> for SupportRequestThread {
    fn from(thread: SupportThread) -> Self {
        let summary = SupportRequestSummary::from(thread.request);
        Self {
            id: summary.id,
            category: summary.category,
            status: summary.status,
            has_screenshot: summary.has_screenshot,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
            messages: thread.messages.into_iter().map(Into::into).collect(),
        }
    }
}

/// Which page of the account's requests to read.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SupportQuery {
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl SupportQuery {
    /// The page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a cursor that does not decode.
    pub fn page(&self) -> Result<PageRequest, Problem> {
        let cursor = self.cursor.as_deref().map(str::parse::<Cursor>);
        Ok(PageRequest::new(cursor.transpose()?, self.limit))
    }
}

/// A reply of the account to one of its requests.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportReplyRequest {
    /// The reply: 1 to 5,000 characters once trimmed.
    pub body: String,
}

/// `at` in RFC 3339.
fn timestamp(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}
