//! What the admin use cases need from outside, besides the accounts, the library, the mailer,
//! the clock and the product events: the server implements them on Postgres. Every change a
//! store makes writes its audit entry in the same transaction, so that there is never one without
//! the other.

mod audit_log;
mod metrics_source;
mod request_state;
mod support_store;
mod user_store;

use thiserror::Error;

pub use audit_log::AuditLog;
pub use metrics_source::{
    DayCount, ExportCount, MetricsSource, Period, ProductAggregates, SizeClassCount, StatusCount,
    SupportAggregates, ToolCount,
};
pub use request_state::RequestState;
pub use support_store::{
    AdminMessage, AdminRequest, AdminSupportStore, AdminThread, QueueFilter, Recipient,
    RequestChange, TeamMessage,
};
pub use user_store::{
    AdminUserStore, Erasure, EventCount, StatusChange, UserDetail, UserFilter, UserSession,
    UserSummary, erased_state, status_state,
};

/// A store failure; the detail is for the logs, and never holds an address or a message.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("admin storage unavailable: {0}")]
pub struct AdminStoreError(pub String);
