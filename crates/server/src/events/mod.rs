//! Product events (H13): [`PostgresEventSink`] batches them to Postgres; [`queries`] computes the
//! aggregates the internal admin API shows; [`spawn_purge`] removes events older than 13 months.

pub mod metrics;
mod purge;
pub mod queries;
mod sink;

pub use purge::{RETENTION_DAYS, purge, spawn_purge};
pub use sink::{APP_VERSION_PROPERTY, LANGUAGE_PROPERTY, PLATFORM_PROPERTY, PostgresEventSink};
