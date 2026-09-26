//! The product metrics: H13's aggregates of the product events, and the support queue's.

use async_trait::async_trait;
use time::OffsetDateTime;

use super::AdminStoreError;
use crate::support::SupportStatus;

/// A half-open period `[since, until)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Period {
    /// Its start, included.
    pub since: OffsetDateTime,
    /// Its end, excluded.
    pub until: OffsetDateTime,
}

/// How many of something happened in a day, a week or a month.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayCount {
    /// The period's first day, at midnight UTC.
    pub day: OffsetDateTime,
    /// How many.
    pub count: i64,
}

/// How many exports of one format came from one source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportCount {
    /// The format.
    pub format: String,
    /// `app` or `mcp`.
    pub source: String,
    /// How many.
    pub count: i64,
}

/// How many calls of one MCP tool had one outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCount {
    /// The tool.
    pub tool: String,
    /// `ok` or `error`.
    pub outcome: String,
    /// How many.
    pub count: i64,
}

/// How many accounts use storage within one size class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SizeClassCount {
    /// The class, as product events name them.
    pub size_class: String,
    /// How many accounts.
    pub count: i64,
}

/// The aggregates of the product events over a period (H13).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProductAggregates {
    /// Sign-ups per day.
    pub signups_per_day: Vec<DayCount>,
    /// Accounts whose first export falls in the period.
    pub activated_accounts: i64,
    /// Distinct active accounts per day.
    pub active_per_day: Vec<DayCount>,
    /// Distinct active accounts per week.
    pub active_per_week: Vec<DayCount>,
    /// Distinct active accounts per month.
    pub active_per_month: Vec<DayCount>,
    /// Exports by format and source.
    pub exports: Vec<ExportCount>,
    /// MCP calls by tool and outcome.
    pub mcp_calls: Vec<ToolCount>,
    /// Accounts by storage size class, now.
    pub storage: Vec<SizeClassCount>,
}

/// How many requests have one status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusCount {
    /// The status.
    pub status: SupportStatus,
    /// How many requests.
    pub count: i64,
}

/// The support queue's measures (docs/admin-console.md, "Support requests").
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupportAggregates {
    /// The open requests — new, in progress, waiting for the user — by status, now.
    pub open_by_status: Vec<StatusCount>,
    /// The median age of the open requests, now, in seconds.
    pub median_age_seconds: Option<f64>,
    /// The median time to the first answer of the requests first answered in the period.
    pub median_first_response_seconds: Option<f64>,
    /// The median time to resolution of the requests resolved in the period.
    pub median_resolution_seconds: Option<f64>,
}

/// Computes the product metrics.
#[async_trait]
pub trait MetricsSource: Send + Sync {
    /// H13's aggregates over `period`.
    async fn product(&self, period: Period) -> Result<ProductAggregates, AdminStoreError>;

    /// The support queue's measures: its state at `now`, its times over `period`.
    async fn support(
        &self,
        period: Period,
        now: OffsetDateTime,
    ) -> Result<SupportAggregates, AdminStoreError>;
}
