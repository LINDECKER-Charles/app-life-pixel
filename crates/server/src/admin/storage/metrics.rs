//! `PostgresMetricsSource`: H13's aggregates of `events::queries`, and the support queue's
//! measures — the open requests by status, the medians of their age, of the time to a first
//! answer and of the time to resolution.

use async_trait::async_trait;
use life_pixel_service::admin::ports::{
    AdminStoreError, DayCount, ExportCount, MetricsSource, Period, ProductAggregates,
    SizeClassCount, StatusCount, SupportAggregates, ToolCount,
};
use life_pixel_service::support::SupportStatus;
use sqlx::PgPool;
use time::OffsetDateTime;

use super::rows::database;
use crate::events::queries::{self, DailyCount};
use crate::storage::metrics::timed;

/// The statuses of the requests still open.
const OPEN_STATUSES: [SupportStatus; 3] = [
    SupportStatus::New,
    SupportStatus::InProgress,
    SupportStatus::WaitingForUser,
];

const OPEN_BY_STATUS: &str = "select status, count(*) from support_requests \
                              where status = any($1) group by status";
const MEDIAN_AGE: &str = "select percentile_cont(0.5) within group \
                          (order by extract(epoch from $2 - created_at)::double precision) \
                          from support_requests where status = any($1)";
const MEDIAN_FIRST_RESPONSE: &str = "select percentile_cont(0.5) within group \
     (order by extract(epoch from first_response_at - created_at)::double precision) \
     from support_requests where first_response_at >= $1 and first_response_at < $2";
const MEDIAN_RESOLUTION: &str = "select percentile_cont(0.5) within group \
     (order by extract(epoch from resolved_at - created_at)::double precision) \
     from support_requests where resolved_at >= $1 and resolved_at < $2";

/// The product metrics of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresMetricsSource {
    pool: PgPool,
}

impl PostgresMetricsSource {
    /// The metrics of the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn open_by_status(&self) -> Result<Vec<StatusCount>, AdminStoreError> {
        let statuses = OPEN_STATUSES.map(SupportStatus::as_str);
        let query = sqlx::query_as::<_, (String, i64)>(OPEN_BY_STATUS).bind(&statuses[..]);
        let rows = timed("admin_open_requests", query.fetch_all(&self.pool)).await;
        let rows = rows.map_err(database)?;
        let count_of = |status: SupportStatus| {
            let row = rows.iter().find(|(name, _)| name == status.as_str());
            row.map_or(0, |(_, count)| *count)
        };
        let counts = OPEN_STATUSES.map(|status| StatusCount {
            status,
            count: count_of(status),
        });
        Ok(counts.to_vec())
    }

    async fn median_age(&self, now: OffsetDateTime) -> Result<Option<f64>, AdminStoreError> {
        let statuses = OPEN_STATUSES.map(SupportStatus::as_str);
        let query = sqlx::query_scalar::<_, Option<f64>>(MEDIAN_AGE)
            .bind(&statuses[..])
            .bind(now);
        let median = timed("admin_median_age", query.fetch_one(&self.pool)).await;
        median.map_err(database)
    }

    async fn median_over(
        &self,
        (name, sql): (&'static str, &'static str),
        period: Period,
    ) -> Result<Option<f64>, AdminStoreError> {
        let query = sqlx::query_scalar::<_, Option<f64>>(sql)
            .bind(period.since)
            .bind(period.until);
        let median = timed(name, query.fetch_one(&self.pool)).await;
        median.map_err(database)
    }
}

/// The day counts of H13's query.
fn days(counts: Vec<DailyCount>) -> Vec<DayCount> {
    let days = counts.into_iter().map(|count| DayCount {
        day: count.day,
        count: count.count,
    });
    days.collect()
}

/// Sign-ups, activation and active accounts over `period`.
async fn activity(pool: &PgPool, period: queries::Period) -> sqlx::Result<ProductAggregates> {
    Ok(ProductAggregates {
        signups_per_day: days(queries::signups_per_day(pool, period).await?),
        activated_accounts: queries::activated_accounts(pool, period).await?,
        active_per_day: days(queries::active_subjects_per_day(pool, period).await?),
        active_per_week: days(queries::active_subjects_per_week(pool, period).await?),
        active_per_month: days(queries::active_subjects_per_month(pool, period).await?),
        ..ProductAggregates::default()
    })
}

/// Exports, MCP calls over `period`, and storage now, beside `activity`.
async fn usage(
    pool: &PgPool,
    (period, activity): (queries::Period, ProductAggregates),
) -> sqlx::Result<ProductAggregates> {
    let exports = queries::exports_by_format_and_source(pool, period).await?;
    let mcp_calls = queries::mcp_calls_by_tool_and_outcome(pool, period).await?;
    let storage = queries::storage_by_size_class(pool).await?;
    let exports = exports.into_iter().map(|row| ExportCount {
        format: row.format,
        source: row.source,
        count: row.count,
    });
    let mcp_calls = mcp_calls.into_iter().map(|row| ToolCount {
        tool: row.tool,
        outcome: row.outcome,
        count: row.count,
    });
    let storage = storage.into_iter().map(|row| SizeClassCount {
        size_class: row.size_class,
        count: row.count,
    });
    Ok(ProductAggregates {
        exports: exports.collect(),
        mcp_calls: mcp_calls.collect(),
        storage: storage.collect(),
        ..activity
    })
}

#[async_trait]
impl MetricsSource for PostgresMetricsSource {
    async fn product(&self, period: Period) -> Result<ProductAggregates, AdminStoreError> {
        let period = queries::Period {
            since: period.since,
            until: period.until,
        };
        let aggregates = timed("admin_product_activity", activity(&self.pool, period)).await;
        let aggregates = aggregates.map_err(database)?;
        let aggregates = usage(&self.pool, (period, aggregates)).await;
        aggregates.map_err(database)
    }

    async fn support(
        &self,
        period: Period,
        now: OffsetDateTime,
    ) -> Result<SupportAggregates, AdminStoreError> {
        let first_response = ("admin_median_first_response", MEDIAN_FIRST_RESPONSE);
        let resolution = ("admin_median_resolution", MEDIAN_RESOLUTION);
        Ok(SupportAggregates {
            open_by_status: self.open_by_status().await?,
            median_age_seconds: self.median_age(now).await?,
            median_first_response_seconds: self.median_over(first_response, period).await?,
            median_resolution_seconds: self.median_over(resolution, period).await?,
        })
    }
}
