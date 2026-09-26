//! The audit log's bodies: an entry, and which entries to list.

use life_pixel_service::admin::{AuditFilter, AuditRecord};
use life_pixel_service::paging::PageRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use super::{non_blank, page, timestamp};
use crate::http::problem::Problem;

/// An entry of the audit log: who did what, to what, when, why, and the state before and after.
#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntryBody {
    /// Its id: later entries have larger ones.
    pub id: i64,
    /// The admin's id.
    pub admin_id: String,
    /// The admin's address.
    pub admin_email: String,
    /// What they did: `user.suspend`, `user.reactivate`, `user.export`, `user.delete`,
    /// `support.update`, `support.reply` or `support.note`.
    pub action: String,
    /// The kind of target: `user` or `support_request`.
    pub target_type: String,
    /// The target's id.
    pub target_id: String,
    /// Why, for the actions on an account.
    pub reason: Option<String>,
    /// The state before the action.
    #[schema(value_type = Option<Object>)]
    pub before: Option<Value>,
    /// The state after the action.
    #[schema(value_type = Option<Object>)]
    pub after: Option<Value>,
    /// When.
    #[schema(format = DateTime)]
    pub at: String,
}

impl From<AuditRecord> for AuditEntryBody {
    fn from(record: AuditRecord) -> Self {
        let entry = record.entry;
        Self {
            id: record.id,
            admin_id: entry.admin_id,
            admin_email: entry.admin_email,
            action: entry.action,
            target_type: entry.target_type,
            target_id: entry.target_id,
            reason: entry.reason,
            before: entry.before,
            after: entry.after,
            at: timestamp(entry.at),
        }
    }
}

/// Which entries to list, newest first.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct AuditQuery {
    /// Only the entries of this admin id.
    pub admin_id: Option<String>,
    /// Only this action.
    pub action: Option<String>,
    /// Only the entries about this target id.
    pub target_id: Option<String>,
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl AuditQuery {
    /// The filter and the page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a cursor that does not decode.
    pub fn filter_and_page(&self) -> Result<(AuditFilter, PageRequest), Problem> {
        let filter = AuditFilter {
            admin_id: non_blank(self.admin_id.as_ref()),
            action: non_blank(self.action.as_ref()),
            target_id: non_blank(self.target_id.as_ref()),
        };
        Ok((filter, page(self.cursor.as_deref(), self.limit)?))
    }
}
