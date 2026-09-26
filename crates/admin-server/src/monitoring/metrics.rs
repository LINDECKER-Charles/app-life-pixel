//! VictoriaMetrics, read through its query-only proxy: `/api/v1/query` for a tile's value at an
//! instant, `/api/v1/query_range` for a panel's series. Nothing else of its API is ever called.

use std::collections::BTreeMap;

use serde::Deserialize;
use time::{Duration, OffsetDateTime};

use super::schema::Series;
use super::source::{Endpoint, Source, SourceError};
use super::time_range::{TimeRange, promql_duration, unix_seconds};
use crate::config::ServiceUrl;

const INSTANT_QUERY_PATH: &str = "/api/v1/query";
const RANGE_QUERY_PATH: &str = "/api/v1/query_range";
const SUCCESS: &str = "success";

/// VictoriaMetrics's answer to a query, in Prometheus's shape.
#[derive(Deserialize)]
struct Answer {
    status: String,
    #[serde(default)]
    data: Option<AnswerData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnswerData {
    #[serde(default)]
    result: Vec<ResultSeries>,
}

/// One series of a vector or a matrix.
#[derive(Deserialize)]
struct ResultSeries {
    #[serde(default)]
    metric: BTreeMap<String, String>,
    #[serde(default)]
    value: Option<(f64, String)>,
    #[serde(default)]
    values: Vec<(f64, String)>,
}

/// Where a series is read: over `range`, a point every `step`, its times moved by `shift` to
/// lay the previous period over the current one.
#[derive(Clone, Copy, Debug)]
pub struct Window {
    /// The period read.
    pub range: TimeRange,
    /// Between two points.
    pub step: Duration,
    /// Added to every time.
    pub shift: Duration,
}

/// A VictoriaMetrics to query.
#[derive(Clone, Debug)]
pub struct MetricsSource<'a> {
    /// The client.
    pub client: &'a reqwest::Client,
    /// `LPA_VICTORIAMETRICS_URL`.
    pub url: &'a ServiceUrl,
}

impl MetricsSource<'_> {
    /// The sum of the values `promql` gives at `time`, or `None` when it gives no number.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn value_at(
        &self,
        promql: &str,
        time: OffsetDateTime,
    ) -> Result<Option<f64>, SourceError> {
        let query = [("query", promql.to_owned()), ("time", unix_seconds(time))];
        let series = self.ask(INSTANT_QUERY_PATH, &query).await?;
        let values: Vec<f64> = series
            .iter()
            .filter_map(|series| series.value.as_ref())
            .filter_map(|(_, value)| number(value))
            .collect();
        Ok((!values.is_empty()).then(|| values.iter().sum()))
    }

    /// The series `promql` gives over the window, named `name`.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn series(
        &self,
        (name, promql): (&str, &str),
        window: Window,
    ) -> Result<Vec<Series>, SourceError> {
        let Window { range, step, shift } = window;
        let query = [
            ("query", promql.to_owned()),
            ("start", unix_seconds(range.from)),
            ("end", unix_seconds(range.to)),
            ("step", promql_duration(step)),
        ];
        let found = self.ask(RANGE_QUERY_PATH, &query).await?;
        let shift_seconds = shift.as_seconds_f64();
        Ok(found
            .into_iter()
            .map(|series| Series {
                name: name.to_owned(),
                labels: series.metric,
                times: series
                    .values
                    .iter()
                    .map(|(time, _)| time + shift_seconds)
                    .collect(),
                values: series
                    .values
                    .iter()
                    .map(|(_, value)| number(value))
                    .collect(),
            })
            .collect())
    }

    async fn ask(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Vec<ResultSeries>, SourceError> {
        let endpoint = Endpoint {
            client: self.client,
            source: Source::VictoriaMetrics,
            url: self.url.join(path),
        };
        let answer: Answer = endpoint.get_json(query).await?;
        if answer.status != SUCCESS {
            return Err(SourceError::unavailable(
                Source::VictoriaMetrics,
                format!("status {}", answer.status),
            ));
        }
        Ok(answer.data.map(|data| data.result).unwrap_or_default())
    }
}

/// A sample's value: `None` for `NaN` and the infinities, which JSON cannot carry.
fn number(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}
