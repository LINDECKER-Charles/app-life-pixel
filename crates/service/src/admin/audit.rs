//! The audit log's entries: who did what, to what, when, why, and the state before and after.
//! No entry ever holds a person's address or a message's text, so that an erasure leaves nothing
//! of them in a log that cannot be changed.

use serde_json::Value;
use time::OffsetDateTime;

use super::values::{AdminIdentity, Reason};

/// What an admin did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuditAction {
    /// An account suspended; its sessions ended.
    UserSuspend,
    /// An account reactivated.
    UserReactivate,
    /// An account's data export read: the right of access.
    UserExport,
    /// An account erased: the right to erasure.
    UserDelete,
    /// A request's status or assignee changed.
    SupportUpdate,
    /// A reply sent to the person who asked.
    SupportReply,
    /// An internal note added to a request.
    SupportNote,
}

impl AuditAction {
    /// The action's value in the log.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserSuspend => "user.suspend",
            Self::UserReactivate => "user.reactivate",
            Self::UserExport => "user.export",
            Self::UserDelete => "user.delete",
            Self::SupportUpdate => "support.update",
            Self::SupportReply => "support.reply",
            Self::SupportNote => "support.note",
        }
    }

    /// The kind of target the action acts on.
    #[must_use]
    pub const fn target_type(self) -> &'static str {
        match self {
            Self::UserSuspend | Self::UserReactivate | Self::UserExport | Self::UserDelete => {
                USER_TARGET
            }
            Self::SupportUpdate | Self::SupportReply | Self::SupportNote => SUPPORT_REQUEST_TARGET,
        }
    }
}

/// The target type of the actions on an account.
pub const USER_TARGET: &str = "user";
/// The target type of the actions on a support request.
pub const SUPPORT_REQUEST_TARGET: &str = "support_request";

/// An action about to be written, before the store reads the state it changes: the store adds
/// the state before and after, in the transaction of the change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditDraft {
    /// Who acts.
    pub admin: AdminIdentity,
    /// What they do.
    pub action: AuditAction,
    /// The id of what they act on.
    pub target_id: String,
    /// Why, for the actions on an account.
    pub reason: Option<Reason>,
    /// When.
    pub at: OffsetDateTime,
}

impl AuditDraft {
    /// The entry of the draft, with the state `before` and `after` the change.
    #[must_use]
    pub fn entry(&self, before: Option<Value>, after: Option<Value>) -> AuditEntry {
        AuditEntry {
            admin_id: self.admin.id().to_owned(),
            admin_email: self.admin.email().to_owned(),
            action: self.action.as_str().to_owned(),
            target_type: self.action.target_type().to_owned(),
            target_id: self.target_id.clone(),
            reason: self
                .reason
                .as_ref()
                .map(|reason| reason.as_str().to_owned()),
            before,
            after,
            at: self.at,
        }
    }
}

/// An entry of the audit log, as it is written and read.
#[derive(Clone, Debug, PartialEq)]
pub struct AuditEntry {
    /// The admin's id.
    pub admin_id: String,
    /// The admin's address.
    pub admin_email: String,
    /// What they did, as [`AuditAction::as_str`] writes it.
    pub action: String,
    /// The kind of target: [`USER_TARGET`] or [`SUPPORT_REQUEST_TARGET`].
    pub target_type: String,
    /// The target's id.
    pub target_id: String,
    /// Why.
    pub reason: Option<String>,
    /// The state before the action.
    pub before: Option<Value>,
    /// The state after the action.
    pub after: Option<Value>,
    /// When.
    pub at: OffsetDateTime,
}

/// An entry as the log keeps it, with its place in the log.
#[derive(Clone, Debug, PartialEq)]
pub struct AuditRecord {
    /// Its id: later entries have larger ones.
    pub id: i64,
    /// The entry.
    pub entry: AuditEntry,
}

/// Which entries to list; every filter set must match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditFilter {
    /// Only the entries of this admin.
    pub admin_id: Option<String>,
    /// Only this action.
    pub action: Option<String>,
    /// Only the entries about this target.
    pub target_id: Option<String>,
}
