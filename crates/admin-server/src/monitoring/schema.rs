//! The bodies of the monitoring routes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::panels::Panel;

/// One tile of the overview: a panel's value at the end of the range, and at the end of the
/// previous one; `null` when there is no data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    /// The panel.
    pub panel: Panel,
    /// Its value: the sum of its series; `latency` is p95.
    pub value: Option<f64>,
    /// Its value one range earlier.
    pub previous: Option<f64>,
}

/// The overview of an environment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    /// The environment.
    pub environment: String,
    /// The range's start, in Unix seconds.
    pub from: i64,
    /// The range's end, in Unix seconds.
    pub to: i64,
    /// The tiles: `up`, `errors`, `latency`, `requests`, `exports`, `mcp`, `storage`.
    pub tiles: Vec<Tile>,
}

/// One series of a panel, as uPlot draws it: its times and values side by side.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    /// The query's name: the panel's, or `p50`, `p95`, `p99`.
    pub name: String,
    /// The series' labels, such as `route` or `format`.
    pub labels: BTreeMap<String, String>,
    /// Unix seconds.
    pub times: Vec<f64>,
    /// The value at each time; `null` where there is none.
    pub values: Vec<Option<f64>>,
}

/// A panel's series over the range, and the previous period's, moved onto the range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PanelSeries {
    /// The panel.
    pub panel: Panel,
    /// The environment.
    pub environment: String,
    /// The range's start, in Unix seconds.
    pub from: i64,
    /// The range's end, in Unix seconds.
    pub to: i64,
    /// Seconds between two points.
    pub step_seconds: i64,
    /// The series.
    pub series: Vec<Series>,
    /// The previous period's series, their times moved forward by the range's length.
    pub previous: Vec<Series>,
}

/// One log line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    /// When, in RFC 3339.
    pub time: String,
    /// Its level, when it has one.
    pub level: Option<String>,
    /// Its message.
    pub message: String,
    /// The request it belongs to, when it names one.
    pub request_id: Option<String>,
    /// Its other fields.
    pub fields: BTreeMap<String, String>,
}

/// Log lines, newest first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogLines {
    /// The lines, `limit` at most.
    pub lines: Vec<LogLine>,
}

/// A firing alert.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// Alertmanager's fingerprint.
    pub fingerprint: String,
    /// `alertname`.
    pub name: String,
    /// `severity`, when set.
    pub severity: Option<String>,
    /// `active`, `suppressed` or `unprocessed`.
    pub state: String,
    /// When it started firing, in RFC 3339.
    pub starts_at: String,
    /// The `summary` annotation.
    pub summary: Option<String>,
    /// The `description` annotation.
    pub description: Option<String>,
    /// Every label.
    pub labels: BTreeMap<String, String>,
}

/// The firing alerts of an environment, newest first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Alerts {
    /// The alerts.
    pub alerts: Vec<Alert>,
}

/// A link to Grafana.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaLink {
    /// Grafana Explore, split between the request's logs and its trace.
    pub url: String,
}
