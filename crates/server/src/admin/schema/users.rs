//! The users' bodies: a user as listed and in detail, the search, and the reason of an action.

use life_pixel_service::accounts::ports::AccountStatus;
use life_pixel_service::admin::ports::{
    EventCount, UserDetail, UserFilter, UserSession, UserSummary,
};
use life_pixel_service::paging::PageRequest;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{non_blank, page, timestamp};
use crate::http::problem::Problem;
use crate::routes::support::schema::SupportRequestSummary;

/// Whether an account may sign in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdminUserStatus {
    /// It may.
    Active,
    /// An admin suspended it.
    Suspended,
}

impl From<AccountStatus> for AdminUserStatus {
    fn from(status: AccountStatus) -> Self {
        match status {
            AccountStatus::Active => Self::Active,
            AccountStatus::Suspended => Self::Suspended,
        }
    }
}

impl From<AdminUserStatus> for AccountStatus {
    fn from(status: AdminUserStatus) -> Self {
        match status {
            AdminUserStatus::Active => Self::Active,
            AdminUserStatus::Suspended => Self::Suspended,
        }
    }
}

/// A user as the search lists them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    /// The account's id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Its address.
    pub email: String,
    /// Whether the address is verified.
    pub email_verified: bool,
    /// Its plan.
    pub plan: String,
    /// Whether it may sign in.
    pub status: AdminUserStatus,
    /// The bytes of its documents.
    pub storage_used_bytes: u64,
    /// When it signed up.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When one of its sessions was last seen; `null` without a session.
    #[schema(format = DateTime)]
    pub last_seen_at: Option<String>,
}

impl From<UserSummary> for AdminUser {
    fn from(user: UserSummary) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            email_verified: user.is_email_verified,
            plan: user.plan,
            status: user.status.into(),
            storage_used_bytes: user.storage_used_bytes,
            created_at: timestamp(user.created_at),
            last_seen_at: user.last_seen_at.map(timestamp),
        }
    }
}

/// How many product events of one name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventCountBody {
    /// The event's name.
    pub name: String,
    /// How many.
    pub count: u64,
}

/// A session of the user, without its token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSession {
    /// When it opened.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it was last seen.
    #[schema(format = DateTime)]
    pub last_seen_at: String,
    /// When it ends, unless it is seen again.
    #[schema(format = DateTime)]
    pub expires_at: String,
}

impl From<UserSession> for AdminSession {
    fn from(session: UserSession) -> Self {
        Self {
            created_at: timestamp(session.created_at),
            last_seen_at: timestamp(session.last_seen_at),
            expires_at: timestamp(session.expires_at),
        }
    }
}

/// A user in detail: as listed, plus the language, the counts of the library, the product
/// events of the last 30 days by name, the sessions and the support requests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserDetail {
    /// The account's id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Its address.
    pub email: String,
    /// Whether the address is verified.
    pub email_verified: bool,
    /// Its plan.
    pub plan: String,
    /// Whether it may sign in.
    pub status: AdminUserStatus,
    /// The bytes of its documents.
    pub storage_used_bytes: u64,
    /// When it signed up.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When one of its sessions was last seen; `null` without a session.
    #[schema(format = DateTime)]
    pub last_seen_at: Option<String>,
    /// The language of its interface and emails.
    pub language: String,
    /// How many projects it has.
    pub project_count: u64,
    /// How many animations it has.
    pub animation_count: u64,
    /// Its product events of the last 30 days, by name.
    pub events: Vec<EventCountBody>,
    /// Its sessions, the most recently seen first.
    pub sessions: Vec<AdminSession>,
    /// Its support requests, the most recently updated first.
    pub support_requests: Vec<SupportRequestSummary>,
}

impl From<UserDetail> for AdminUserDetail {
    fn from(detail: UserDetail) -> Self {
        let user = AdminUser::from(detail.summary);
        let events = detail
            .events
            .into_iter()
            .map(|EventCount { name, count }| EventCountBody { name, count });
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            plan: user.plan,
            status: user.status,
            storage_used_bytes: user.storage_used_bytes,
            created_at: user.created_at,
            last_seen_at: user.last_seen_at,
            language: detail.language,
            project_count: detail.project_count,
            animation_count: detail.animation_count,
            events: events.collect(),
            sessions: detail.sessions.into_iter().map(Into::into).collect(),
            support_requests: detail
                .support_requests
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// Which users to list, from the most recent sign-up.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UserQuery {
    /// Part of the address, whatever its case, or the whole account id.
    pub q: Option<String>,
    /// Only the users of this status.
    pub status: Option<AdminUserStatus>,
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl UserQuery {
    /// The filter and the page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a cursor that does not decode.
    pub fn filter_and_page(&self) -> Result<(UserFilter, PageRequest), Problem> {
        let filter = UserFilter {
            query: non_blank(self.q.as_ref()),
            status: self.status.map(Into::into),
        };
        Ok((filter, page(self.cursor.as_deref(), self.limit)?))
    }
}

/// Why an admin acts on an account: kept in the audit log.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReasonBody {
    /// The reason: 1 to 1,000 characters once trimmed.
    pub reason: String,
}
