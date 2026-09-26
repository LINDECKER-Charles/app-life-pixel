//! The product metrics' bodies: H13's aggregates over a period, and the support queue's measures.

use life_pixel_service::admin::ProductMetrics;
use life_pixel_service::admin::ports::{
    DayCount, ExportCount, ProductAggregates, SizeClassCount, StatusCount, SupportAggregates,
    ToolCount,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};

use super::timestamp;
use crate::http::problem::{Problem, codes};
use crate::routes::support::schema::SupportRequestStatus;

/// A count of one day, week or month, named by its first day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DayCountBody {
    /// Its first day, at midnight UTC.
    #[schema(format = DateTime)]
    pub day: String,
    /// How many.
    pub count: i64,
}

impl From<DayCount> for DayCountBody {
    fn from(count: DayCount) -> Self {
        Self {
            day: timestamp(count.day),
            count: count.count,
        }
    }
}

/// How many exports of one format from one source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportCountBody {
    /// `wasm`, `gif`, `apng`, `sprite_sheet` or `png_frames`.
    pub format: String,
    /// `app` or `mcp`.
    pub source: String,
    /// How many.
    pub count: i64,
}

/// How many MCP calls of one tool with one outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCountBody {
    /// The tool's name.
    pub tool: String,
    /// `ok` or `error`.
    pub outcome: String,
    /// How many.
    pub count: i64,
}

/// How many accounts in one class of storage used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SizeClassCountBody {
    /// `lt_1k`, `lt_10k`, `lt_100k`, `lt_1m` or `ge_1m`.
    pub size_class: String,
    /// How many accounts.
    pub count: i64,
}

/// H13's aggregates over the period.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductAggregatesBody {
    /// The sign-ups of each day.
    pub signups_per_day: Vec<DayCountBody>,
    /// The accounts whose first export falls in the period.
    pub activated_accounts: i64,
    /// The active accounts of each day.
    pub active_per_day: Vec<DayCountBody>,
    /// The active accounts of each week.
    pub active_per_week: Vec<DayCountBody>,
    /// The active accounts of each month.
    pub active_per_month: Vec<DayCountBody>,
    /// The exports by format and source.
    pub exports: Vec<ExportCountBody>,
    /// The MCP calls by tool and outcome.
    pub mcp_calls: Vec<ToolCountBody>,
    /// The accounts by class of storage used, now.
    pub storage: Vec<SizeClassCountBody>,
}

impl From<ProductAggregates> for ProductAggregatesBody {
    fn from(product: ProductAggregates) -> Self {
        let days = |counts: Vec<DayCount>| counts.into_iter().map(Into::into).collect();
        Self {
            signups_per_day: days(product.signups_per_day),
            activated_accounts: product.activated_accounts,
            active_per_day: days(product.active_per_day),
            active_per_week: days(product.active_per_week),
            active_per_month: days(product.active_per_month),
            exports: product.exports.into_iter().map(Into::into).collect(),
            mcp_calls: product.mcp_calls.into_iter().map(Into::into).collect(),
            storage: product.storage.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ExportCount> for ExportCountBody {
    fn from(row: ExportCount) -> Self {
        Self {
            format: row.format,
            source: row.source,
            count: row.count,
        }
    }
}

impl From<ToolCount> for ToolCountBody {
    fn from(row: ToolCount) -> Self {
        Self {
            tool: row.tool,
            outcome: row.outcome,
            count: row.count,
        }
    }
}

impl From<SizeClassCount> for SizeClassCountBody {
    fn from(row: SizeClassCount) -> Self {
        Self {
            size_class: row.size_class,
            count: row.count,
        }
    }
}

/// How many open requests of one status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusCountBody {
    /// The status: `new`, `in_progress` or `waiting_for_user`.
    pub status: SupportRequestStatus,
    /// How many requests.
    pub count: i64,
}

/// The support queue's measures.
#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportMetricsBody {
    /// The open requests by status, now.
    pub open_by_status: Vec<StatusCountBody>,
    /// The median age of the open requests now, in seconds; `null` without one.
    pub median_age_seconds: Option<f64>,
    /// The median time to a first answer of the requests first answered over the period, in
    /// seconds; `null` without one.
    pub median_first_response_seconds: Option<f64>,
    /// The median time to resolution of the requests resolved over the period, in seconds;
    /// `null` without one.
    pub median_resolution_seconds: Option<f64>,
}

impl From<SupportAggregates> for SupportMetricsBody {
    fn from(support: SupportAggregates) -> Self {
        let open = support
            .open_by_status
            .into_iter()
            .map(|row: StatusCount| StatusCountBody {
                status: row.status.into(),
                count: row.count,
            });
        Self {
            open_by_status: open.collect(),
            median_age_seconds: support.median_age_seconds,
            median_first_response_seconds: support.median_first_response_seconds,
            median_resolution_seconds: support.median_resolution_seconds,
        }
    }
}

/// The product metrics of a period.
#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductMetricsBody {
    /// The period's start, included.
    #[schema(format = DateTime)]
    pub from: String,
    /// The period's end, excluded.
    #[schema(format = DateTime)]
    pub to: String,
    /// H13's aggregates over it.
    pub product: ProductAggregatesBody,
    /// The support queue's measures.
    pub support: SupportMetricsBody,
}

impl From<ProductMetrics> for ProductMetricsBody {
    fn from(metrics: ProductMetrics) -> Self {
        Self {
            from: timestamp(metrics.period.since),
            to: timestamp(metrics.period.until),
            product: metrics.product.into(),
            support: metrics.support.into(),
        }
    }
}

/// The period of the metrics: the 30 days until now by default.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MetricsQuery {
    /// The period's start, included, in RFC 3339; 30 days before its end by default.
    #[param(format = DateTime)]
    pub from: Option<String>,
    /// The period's end, excluded, in RFC 3339; now by default.
    #[param(format = DateTime)]
    pub to: Option<String>,
}

impl MetricsQuery {
    /// The period's bounds that are given.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a bound that is not RFC 3339.
    pub fn bounds(&self) -> Result<(Option<OffsetDateTime>, Option<OffsetDateTime>), Problem> {
        Ok((instant(self.from.as_deref())?, instant(self.to.as_deref())?))
    }
}

fn instant(text: Option<&str>) -> Result<Option<OffsetDateTime>, Problem> {
    let parsed = text.map(|text| OffsetDateTime::parse(text, &Rfc3339));
    parsed
        .transpose()
        .map_err(|_| Problem::new(codes::REQUEST_MALFORMED))
}
