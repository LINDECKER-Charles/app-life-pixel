//! The support queue's bodies: a request as the team sees it, its thread with the internal notes
//! and the context, the queue's filters, and what the team changes and writes.

use life_pixel_service::admin::ports::{AdminMessage, AdminRequest, QueueFilter};
use life_pixel_service::admin::{MessageDraft, QueuedRequest, QueuedThread, RequestUpdate};
use life_pixel_service::paging::PageRequest;
use life_pixel_service::support::{
    Author, Category, SupportContext, SupportRequestId, SupportStatus,
};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{non_blank, page, timestamp};
use crate::http::problem::{Problem, codes};
use crate::routes::support::schema::{SupportAuthor, SupportCategory, SupportRequestStatus};

/// A request as the team sees it, without its messages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSupportRequest {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// The account that sent it.
    #[schema(format = "uuid")]
    pub account_id: String,
    /// That account's address.
    pub email: String,
    /// What it is about.
    pub category: SupportCategory,
    /// Where it stands.
    pub status: SupportRequestStatus,
    /// Whether a screenshot came with it.
    pub has_screenshot: bool,
    /// The admin it is assigned to; `null` for nobody.
    pub assigned_to: Option<String>,
    /// When it was sent.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When its status last changed, or its author or the team last wrote.
    #[schema(format = DateTime)]
    pub updated_at: String,
    /// When the team first replied; `null` before.
    #[schema(format = DateTime)]
    pub first_response_at: Option<String>,
    /// When it was resolved or closed, while it is.
    #[schema(format = DateTime)]
    pub resolved_at: Option<String>,
    /// How long ago it was sent, in seconds.
    pub age_seconds: i64,
}

impl From<QueuedRequest> for AdminSupportRequest {
    fn from(queued: QueuedRequest) -> Self {
        let AdminRequest {
            request,
            account,
            email,
            assigned_to,
            first_response_at,
            resolved_at,
        } = queued.request;
        Self {
            id: request.id.to_string(),
            account_id: account.to_string(),
            email,
            category: request.category.into(),
            status: request.status.into(),
            has_screenshot: request.has_screenshot,
            assigned_to,
            created_at: timestamp(request.created_at),
            updated_at: timestamp(request.updated_at),
            first_response_at: first_response_at.map(timestamp),
            resolved_at: resolved_at.map(timestamp),
            age_seconds: queued.age_seconds,
        }
    }
}

/// What the app attached to a request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSupportContext {
    /// The app's version.
    pub app_version: String,
    /// The app's platform: `web`, `android`…
    pub platform: String,
    /// The interface's language code.
    pub language: String,
    /// The route template of the screen the person was on.
    pub screen: String,
}

impl From<SupportContext> for AdminSupportContext {
    fn from(context: SupportContext) -> Self {
        Self {
            app_version: context.app_version,
            platform: context.platform,
            language: context.language,
            screen: context.screen,
        }
    }
}

/// A message of a request, internal notes included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSupportMessage {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Who wrote it.
    pub author: SupportAuthor,
    /// The admin who wrote it; `null` for the person's messages.
    pub admin_id: Option<String>,
    /// Its text.
    pub body: String,
    /// Whether it is a note of the team the person never sees.
    pub internal: bool,
    /// When it was written.
    #[schema(format = DateTime)]
    pub created_at: String,
}

impl From<AdminMessage> for AdminSupportMessage {
    fn from(message: AdminMessage) -> Self {
        let author = match message.author {
            Author::User => SupportAuthor::User,
            Author::Team => SupportAuthor::Team,
        };
        Self {
            id: message.id.to_string(),
            author,
            admin_id: message.admin_id,
            body: message.body,
            internal: message.is_internal,
            created_at: timestamp(message.created_at),
        }
    }
}

/// A request with its context and every message, oldest first, internal notes included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSupportThread {
    /// The request, with its age.
    pub request: AdminSupportRequest,
    /// Whether a screenshot came with it, served by `GET /support-requests/{id}/screenshot`.
    pub has_screenshot: bool,
    /// What the app attached.
    pub context: AdminSupportContext,
    /// Its messages: the first is the request's own.
    pub messages: Vec<AdminSupportMessage>,
}

impl From<QueuedThread> for AdminSupportThread {
    fn from(
        QueuedThread {
            thread,
            age_seconds,
        }: QueuedThread,
    ) -> Self {
        let request = AdminSupportRequest::from(QueuedRequest {
            request: thread.request,
            age_seconds,
        });
        Self {
            has_screenshot: request.has_screenshot,
            request,
            context: thread.context.into(),
            messages: thread.messages.into_iter().map(Into::into).collect(),
        }
    }
}

/// A request's screenshot, as the PNG it was stored as.
#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct Screenshot(pub Vec<u8>);

/// Which requests the queue lists, from the most recently updated.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueueQuery {
    /// Only the requests of this status.
    #[param(value_type = Option<SupportRequestStatus>)]
    pub status: Option<String>,
    /// Only the requests of this category.
    #[param(value_type = Option<SupportCategory>)]
    pub category: Option<String>,
    /// Only the requests assigned to this admin id.
    pub assignee: Option<String>,
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl QueueQuery {
    /// The filter and the page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for an unknown status or category, or a cursor that does not decode.
    pub fn filter_and_page(&self) -> Result<(QueueFilter, PageRequest), Problem> {
        let category = non_blank(self.category.as_ref());
        let category = category.map(|name| Category::parse(&name).map_err(|_| malformed()));
        let filter = QueueFilter {
            status: non_blank(self.status.as_ref())
                .map(|name| status(&name))
                .transpose()?,
            category: category.transpose()?,
            assignee: non_blank(self.assignee.as_ref()),
        };
        Ok((filter, page(self.cursor.as_deref(), self.limit)?))
    }
}

/// A change of a request: its status, its assignee, or both.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPatch {
    /// Its new status.
    #[schema(value_type = Option<SupportRequestStatus>)]
    pub status: Option<String>,
    /// The admin id it is assigned to; `null` for nobody. Absent, the assignee stays.
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<String>)]
    pub assigned_to: Option<Option<String>>,
}

impl RequestPatch {
    /// The update of the request `id` this patch asks for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for an unknown status.
    pub fn update(self, id: SupportRequestId) -> Result<RequestUpdate, Problem> {
        Ok(RequestUpdate {
            id,
            status: self.status.as_deref().map(status).transpose()?,
            assigned_to: self.assigned_to,
        })
    }
}

/// A message of the team: a reply the person is emailed about, or an internal note.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagePost {
    /// Its text: 1 to 5,000 characters once trimmed.
    pub body: String,
    /// Whether it is a note the person never sees, rather than a reply.
    pub internal: bool,
}

impl MessagePost {
    /// The draft of this message on the request `id`.
    #[must_use]
    pub fn draft(self, id: SupportRequestId) -> MessageDraft {
        MessageDraft {
            request: id,
            body: self.body,
            is_internal: self.internal,
        }
    }
}

/// A field that may be `null`: `Some(None)` when it is, `None` when it is absent.
fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

/// The status `name` spells.
fn status(name: &str) -> Result<SupportStatus, Problem> {
    SupportStatus::parse(name).ok_or_else(malformed)
}

fn malformed() -> Problem {
    Problem::new(codes::REQUEST_MALFORMED)
}
