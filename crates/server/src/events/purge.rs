//! The daily purge of product events older than 13 months (H13).

use sqlx::PgPool;
use time::{Duration as CalendarDuration, OffsetDateTime};
use tokio::time::MissedTickBehavior;

/// How long an event is kept: 395 days, 13 months.
pub const RETENTION_DAYS: i64 = 395;
/// How often the purge runs.
const PURGE_PERIOD: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// Deletes the events older than [`RETENTION_DAYS`], and returns how many.
///
/// # Errors
///
/// When the database does not answer.
pub async fn purge(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let cutoff = OffsetDateTime::now_utc() - CalendarDuration::days(RETENTION_DAYS);
    let deleted = sqlx::query("delete from product_events where occurred_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(deleted.rows_affected())
}

/// Purges now, then every day, on the current runtime.
pub fn spawn_purge(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(PURGE_PERIOD);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            match purge(&pool).await {
                Ok(deleted) => tracing::info!(deleted, "old product events are purged"),
                Err(error) => tracing::warn!(%error, "product events were not purged"),
            }
        }
    });
}
