//! Where the admin reads accounts and acts on them.

use async_trait::async_trait;
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::AdminStoreError;
use crate::accounts::ports::AccountStatus;
use crate::admin::AuditDraft;
use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};
use crate::support::ports::SupportRequest;

/// An account as the admin lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserSummary {
    /// Its id.
    pub id: AccountId,
    /// Its address.
    pub email: String,
    /// Whether the address is verified.
    pub is_email_verified: bool,
    /// Its plan.
    pub plan: String,
    /// Whether it may sign in.
    pub status: AccountStatus,
    /// The bytes of its documents.
    pub storage_used_bytes: u64,
    /// When it signed up.
    pub created_at: OffsetDateTime,
    /// When one of its sessions was last seen, if it has one.
    pub last_seen_at: Option<OffsetDateTime>,
}

/// Which accounts to list, from the most recent sign-up; every filter set must match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserFilter {
    /// Part of the address, whatever its case, or the whole id.
    pub query: Option<String>,
    /// Only the accounts of this status.
    pub status: Option<AccountStatus>,
}

/// How many product events of one name an account made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventCount {
    /// The event's name.
    pub name: String,
    /// How many.
    pub count: u64,
}

/// A session of an account, without its token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserSession {
    /// When it opened.
    pub created_at: OffsetDateTime,
    /// When it was last seen.
    pub last_seen_at: OffsetDateTime,
    /// When it ends, unless it is seen again.
    pub expires_at: OffsetDateTime,
}

/// An account with what the admin needs to help it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserDetail {
    /// The account, as listed.
    pub summary: UserSummary,
    /// Its language.
    pub language: String,
    /// How many projects it has.
    pub project_count: u64,
    /// How many animations it has.
    pub animation_count: u64,
    /// Its product events since the date asked, by name.
    pub events: Vec<EventCount>,
    /// Its sessions, the most recently seen first.
    pub sessions: Vec<UserSession>,
    /// Its support requests, the most recently updated first.
    pub support_requests: Vec<SupportRequest>,
}

/// An account suspended or reactivated: its status, its sessions ended when suspended, and the
/// audit entry, in one transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusChange {
    /// The account.
    pub account: AccountId,
    /// Its new status.
    pub status: AccountStatus,
    /// The action, written with the state before and after.
    pub audit: AuditDraft,
}

/// An account erased, once its documents are gone: the account, and with it everything it had,
/// and the audit entry, in one transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Erasure {
    /// The account.
    pub account: AccountId,
    /// The action, written with the state before — never the address.
    pub audit: AuditDraft,
}

/// Where the admin reads accounts and acts on them.
#[async_trait]
pub trait AdminUserStore: Send + Sync {
    /// A page of the accounts `filter` keeps, from the most recent sign-up.
    async fn search(
        &self,
        filter: UserFilter,
        page: PageRequest,
    ) -> Result<Page<UserSummary>, AdminStoreError>;

    /// The account `id`, if it exists.
    async fn find(&self, id: AccountId) -> Result<Option<UserSummary>, AdminStoreError>;

    /// The account `id` in detail, with its product events since `events_since`.
    async fn detail(
        &self,
        id: AccountId,
        events_since: OffsetDateTime,
    ) -> Result<Option<UserDetail>, AdminStoreError>;

    /// Applies `change` with its audit entry, [`status_state`] before and after; `false` when the
    /// account does not exist, and nothing is written.
    async fn set_status(&self, change: StatusChange) -> Result<bool, AdminStoreError>;

    /// Applies `erasure` with its audit entry, [`erased_state`] before; `false` when the account
    /// does not exist, and nothing is written.
    async fn erase(&self, erasure: Erasure) -> Result<bool, AdminStoreError>;
}

/// The state an audit entry keeps of an account's status.
#[must_use]
pub fn status_state(status: AccountStatus) -> Value {
    json!({ "status": status.as_str() })
}

/// The state an audit entry keeps of an erased account: what it was, never its address.
#[must_use]
pub fn erased_state(user: &UserSummary) -> Value {
    json!({
        "status": user.status.as_str(),
        "plan": user.plan,
        "storageUsedBytes": user.storage_used_bytes,
        "createdAt": user.created_at.format(&Rfc3339).unwrap_or_default(),
    })
}
