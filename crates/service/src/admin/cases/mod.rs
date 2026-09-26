//! The admin use cases, one module per area: the users and what the admin does to them, the
//! support queue and the team's answers, the product metrics and the audit log.

mod account_actions;
mod audit_log;
mod metrics;
mod support;
mod support_changes;
mod users;

pub use metrics::ProductMetrics;
pub use support::{QueuedRequest, QueuedThread};
pub use support_changes::{MessageDraft, RequestUpdate};
