//! The aggregates the internal admin API shows (H10), computed straight from `product_events`
//! and, for storage, from `accounts.storage_used_bytes`.

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;

/// A half-open period `[since, until)` an aggregate is computed over.
#[derive(Clone, Copy, Debug)]
pub struct Period {
    /// The period's start, included.
    pub since: OffsetDateTime,
    /// The period's end, excluded.
    pub until: OffsetDateTime,
}

/// How many events of one name happened on one day.
#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct DailyCount {
    /// The day, at midnight UTC.
    pub day: OffsetDateTime,
    /// How many.
    pub count: i64,
}

/// How many exports happened, by format and source.
#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct FormatSourceCount {
    /// `wasm`, `gif`, `apng`, `sprite_sheet` or `png_frames`.
    pub format: String,
    /// `app` or `mcp`.
    pub source: String,
    /// How many.
    pub count: i64,
}

/// How many MCP calls happened, by tool and outcome.
#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct ToolOutcomeCount {
    /// The tool's name.
    pub tool: String,
    /// `ok` or `error`.
    pub outcome: String,
    /// How many.
    pub count: i64,
}

/// How many accounts fall in one size class of storage used.
#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct SizeClassCount {
    /// `lt_1k`, `lt_10k`, `lt_100k`, `lt_1m` or `ge_1m`, as `ProductEvent::size_class` classes.
    pub size_class: String,
    /// How many.
    pub count: i64,
}

/// Sign-ups per day: `signed_up` events, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn signups_per_day(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<DailyCount>, sqlx::Error> {
    daily_count(pool, "signed_up", period).await
}

/// Accounts whose first `export_completed` falls in `period`: activation.
///
/// # Errors
///
/// When the database does not answer.
pub async fn activated_accounts(pool: &PgPool, period: Period) -> Result<i64, sqlx::Error> {
    const QUERY: &str = "with first_export as ( \
         select subject, min(occurred_at) as first_at from product_events \
         where name = 'export_completed' and subject is not null group by subject \
     ) select count(*) from first_export where first_at >= $1 and first_at < $2";
    sqlx::query_scalar::<_, i64>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_one(pool)
        .await
}

/// Distinct active subjects per day, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn active_subjects_per_day(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<DailyCount>, sqlx::Error> {
    const QUERY: &str = "select date_trunc('day', occurred_at) as day, \
         count(distinct subject) as count from product_events where subject is not null \
         and occurred_at >= $1 and occurred_at < $2 group by day order by day";
    sqlx::query_as::<_, DailyCount>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}

/// Distinct active subjects per week, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn active_subjects_per_week(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<DailyCount>, sqlx::Error> {
    const QUERY: &str = "select date_trunc('week', occurred_at) as day, \
         count(distinct subject) as count from product_events where subject is not null \
         and occurred_at >= $1 and occurred_at < $2 group by day order by day";
    sqlx::query_as::<_, DailyCount>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}

/// Distinct active subjects per month, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn active_subjects_per_month(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<DailyCount>, sqlx::Error> {
    const QUERY: &str = "select date_trunc('month', occurred_at) as day, \
         count(distinct subject) as count from product_events where subject is not null \
         and occurred_at >= $1 and occurred_at < $2 group by day order by day";
    sqlx::query_as::<_, DailyCount>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}

/// Exports by format and source, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn exports_by_format_and_source(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<FormatSourceCount>, sqlx::Error> {
    const QUERY: &str = "select coalesce(properties->>'format', 'unknown') as format, \
         coalesce(properties->>'source', 'unknown') as source, count(*) as count \
         from product_events where name = 'export_completed' \
         and occurred_at >= $1 and occurred_at < $2 group by format, source order by format, source";
    sqlx::query_as::<_, FormatSourceCount>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}

/// MCP calls by tool and outcome, over `period`.
///
/// # Errors
///
/// When the database does not answer.
pub async fn mcp_calls_by_tool_and_outcome(
    pool: &PgPool,
    period: Period,
) -> Result<Vec<ToolOutcomeCount>, sqlx::Error> {
    const QUERY: &str = "select coalesce(properties->>'tool', 'unknown') as tool, \
         coalesce(properties->>'outcome', 'unknown') as outcome, count(*) as count \
         from product_events where name = 'mcp_tool_called' \
         and occurred_at >= $1 and occurred_at < $2 group by tool, outcome order by tool, outcome";
    sqlx::query_as::<_, ToolOutcomeCount>(QUERY)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}

/// Storage used, distributed by size class: the same bounds as
/// `life_pixel_service::ports::ProductEvent::size_class`, applied in SQL to
/// `accounts.storage_used_bytes` since it is not itself a product event.
///
/// # Errors
///
/// When the database does not answer.
pub async fn storage_by_size_class(pool: &PgPool) -> Result<Vec<SizeClassCount>, sqlx::Error> {
    const QUERY: &str = "select case \
         when storage_used_bytes < 1024 then 'lt_1k' \
         when storage_used_bytes < 10240 then 'lt_10k' \
         when storage_used_bytes < 102400 then 'lt_100k' \
         when storage_used_bytes < 1048576 then 'lt_1m' \
         else 'ge_1m' end as size_class, count(*) as count \
         from accounts group by size_class order by min(storage_used_bytes)";
    sqlx::query_as::<_, SizeClassCount>(QUERY)
        .fetch_all(pool)
        .await
}

async fn daily_count(
    pool: &PgPool,
    name: &'static str,
    period: Period,
) -> Result<Vec<DailyCount>, sqlx::Error> {
    const QUERY: &str = "select date_trunc('day', occurred_at) as day, count(*) as count \
         from product_events where name = $1 and occurred_at >= $2 and occurred_at < $3 \
         group by day order by day";
    sqlx::query_as::<_, DailyCount>(QUERY)
        .bind(name)
        .bind(period.since)
        .bind(period.until)
        .fetch_all(pool)
        .await
}
