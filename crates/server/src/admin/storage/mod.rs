//! The admin API's stores on Postgres: the accounts, the support queue, the audit log and the
//! product metrics. Every change writes its audit entry in its own transaction. One line per
//! module.

mod audit;
mod metrics;
mod rows;
mod support;
mod support_changes;
mod user_changes;
mod user_detail;
mod users;

pub use audit::PostgresAuditLog;
pub use metrics::PostgresMetricsSource;
pub use support::PostgresAdminSupportStore;
pub use users::PostgresAdminUserStore;
