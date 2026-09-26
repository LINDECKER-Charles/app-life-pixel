//! Metrics: the Prometheus recorder, the `/metrics` handler, and the `http_*` metrics of the
//! catalogue of docs/admin-console.md. Each feature registers its own, with low-cardinality
//! labels: a route template, never a raw path, an id or an email.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use metrics::{Unit, describe_counter, describe_histogram};
use metrics_exporter_prometheus::{BuildError, Matcher, PrometheusBuilder, PrometheusHandle};

/// Requests answered, by `route`, `method` and `status_class`.
pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
/// Time to answer, by `route` and `method`.
pub const HTTP_REQUEST_DURATION_SECONDS: &str = "http_request_duration_seconds";
/// How often the recorder drains its histograms.
pub const METRICS_UPKEEP_PERIOD: Duration = Duration::from_secs(5);

/// The buckets of the request durations, in seconds.
const DURATION_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];
const PROMETHEUS_TEXT: &str = "text/plain; version=0.0.4; charset=utf-8";

/// The recorder of the process, once installed.
static RECORDER: Mutex<Option<PrometheusHandle>> = Mutex::new(None);

/// The process's metrics recorder, installed on the first call.
///
/// # Errors
///
/// When the recorder cannot be installed.
pub fn recorder() -> Result<PrometheusHandle, BuildError> {
    let mut installed = RECORDER.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(handle) = installed.as_ref() {
        return Ok(handle.clone());
    }
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(HTTP_REQUEST_DURATION_SECONDS.to_owned()),
            DURATION_BUCKETS,
        )?
        .install_recorder()?;
    describe_http_metrics();
    *installed = Some(handle.clone());
    Ok(handle)
}

/// `GET /metrics`: every metric, as Prometheus text.
pub async fn render_metrics(State(handle): State<PrometheusHandle>) -> impl IntoResponse {
    ([(CONTENT_TYPE, PROMETHEUS_TEXT)], handle.render())
}

fn describe_http_metrics() {
    describe_counter!(HTTP_REQUESTS_TOTAL, "HTTP requests answered");
    describe_histogram!(
        HTTP_REQUEST_DURATION_SECONDS,
        Unit::Seconds,
        "Time to answer an HTTP request"
    );
}
