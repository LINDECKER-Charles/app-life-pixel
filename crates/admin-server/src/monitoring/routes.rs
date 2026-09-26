//! `/monitoring/overview`, `/monitoring/series`, `/logs`, `/alerts` and `/links/grafana`: every
//! route takes the environment as `env`, and the time range as `from` and `to`.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::alerts::{AlertsSource, matchers};
use super::links::{RequestLink, explore_url, is_request_id};
use super::logs::{
    DEFAULT_LOG_LINES, LogLevel, LogSearch, LogsRequest, LogsSource, MAX_LOG_LINES, logsql,
};
use super::panels::Panel;
use super::schema::{Alerts, GrafanaLink, LogLines, Overview, PanelSeries};
use super::source::{Source, SourceError};
use super::time_range::TimeRange;
use super::{Monitoring, PanelRequest, configured};
use crate::http::problem::{Problem, ProblemDocument, codes};
use crate::state::AppState;

/// An environment and a time range.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RangeQuery {
    /// One of `LPA_ENVIRONMENTS`.
    env: String,
    /// `now`, `now-<n><unit>` (`s`, `m`, `h`, `d`, `w`), Unix seconds or RFC 3339; `now-24h`.
    from: Option<String>,
    /// The same forms; `now`. At most 90 days after `from`.
    to: Option<String>,
}

/// A panel over a time range.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SeriesQuery {
    /// One of `LPA_ENVIRONMENTS`.
    env: String,
    /// The panel.
    panel: Panel,
    /// As the overview's.
    from: Option<String>,
    /// As the overview's.
    to: Option<String>,
    /// Seconds, or `<n><unit>`, between two points; 200 points when absent, 1,000 at most,
    /// 15 seconds at least.
    step: Option<String>,
}

/// A search of the logs.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogsQuery {
    /// One of `LPA_ENVIRONMENTS`.
    env: String,
    /// Only this level, whatever its case.
    level: Option<LogLevel>,
    /// Only lines holding this text, as one phrase.
    q: Option<String>,
    /// As the overview's.
    from: Option<String>,
    /// As the overview's.
    to: Option<String>,
    /// The newest lines to return: 100 when absent, 500 at most.
    limit: Option<usize>,
}

/// An environment.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EnvironmentQuery {
    /// One of `LPA_ENVIRONMENTS`.
    env: String,
}

/// A request to find in Grafana.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GrafanaQuery {
    /// One of `LPA_ENVIRONMENTS`.
    env: String,
    /// The request's id, as its log lines carry it.
    request_id: String,
    /// As the overview's.
    from: Option<String>,
    /// As the overview's.
    to: Option<String>,
}

/// The monitoring routes.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(overview))
        .routes(routes!(series))
        .routes(routes!(logs))
        .routes(routes!(alerts))
        .routes(routes!(grafana_link))
}

/// Is everything fine? The tiles of an environment.
#[utoipa::path(
    get, path = "/monitoring/overview", tag = "monitoring", operation_id = "getOverview",
    params(RangeQuery),
    responses(
        (status = OK, description = "The tiles", body = Overview),
        (status = BAD_REQUEST, description = "`request.malformed`: an unknown environment or an \
            invalid range", body = ProblemDocument, content_type = "application/problem+json"),
        (status = BAD_GATEWAY, description = "`monitoring.unavailable`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = SERVICE_UNAVAILABLE, description = "`monitoring.not_configured`",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn overview(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<Overview>, Problem> {
    let range = range(&state, query.from.as_deref(), query.to.as_deref())?;
    Ok(Json(state.monitoring.overview(&query.env, range).await?))
}

/// A panel's series, and the previous period's.
#[utoipa::path(
    get, path = "/monitoring/series", tag = "monitoring", operation_id = "getPanelSeries",
    params(SeriesQuery),
    responses(
        (status = OK, description = "The series", body = PanelSeries),
        (status = BAD_REQUEST, description = "`request.malformed`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = BAD_GATEWAY, description = "`monitoring.unavailable`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = SERVICE_UNAVAILABLE, description = "`monitoring.not_configured`",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn series(
    State(state): State<AppState>,
    Query(query): Query<SeriesQuery>,
) -> Result<Json<PanelSeries>, Problem> {
    let range = range(&state, query.from.as_deref(), query.to.as_deref())?;
    let request = PanelRequest {
        panel: query.panel,
        range,
        step: range.step(query.step.as_deref()).ok_or_else(malformed)?,
    };
    Ok(Json(state.monitoring.series(&query.env, request).await?))
}

/// The newest log lines of an environment.
#[utoipa::path(
    get, path = "/logs", tag = "monitoring", operation_id = "getLogs",
    params(LogsQuery),
    responses(
        (status = OK, description = "The lines, newest first", body = LogLines),
        (status = BAD_REQUEST, description = "`request.malformed`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = BAD_GATEWAY, description = "`monitoring.unavailable`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = SERVICE_UNAVAILABLE, description = "`monitoring.not_configured`",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<LogLines>, Problem> {
    let range = range(&state, query.from.as_deref(), query.to.as_deref())?;
    let monitoring = &state.monitoring;
    let selector = logs_selector(monitoring, &query.env)?;
    let url = configured(
        monitoring.config().victoria_logs.as_ref(),
        Source::VictoriaLogs,
    )?;
    let search = LogSearch {
        level: query.level,
        text: query.q,
    };
    let limit = query
        .limit
        .unwrap_or(DEFAULT_LOG_LINES)
        .clamp(1, MAX_LOG_LINES);
    let source = LogsSource {
        client: monitoring.client(),
        url,
    };
    let request = LogsRequest {
        query: logsql(selector, &search),
        range,
        limit,
    };
    let lines = source.lines(request).await?;
    Ok(Json(LogLines { lines }))
}

/// The firing alerts of an environment.
#[utoipa::path(
    get, path = "/alerts", tag = "monitoring", operation_id = "getAlerts",
    params(EnvironmentQuery),
    responses(
        (status = OK, description = "The alerts, newest first", body = Alerts),
        (status = BAD_REQUEST, description = "`request.malformed`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = BAD_GATEWAY, description = "`monitoring.unavailable`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = SERVICE_UNAVAILABLE, description = "`monitoring.not_configured`",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn alerts(
    State(state): State<AppState>,
    Query(query): Query<EnvironmentQuery>,
) -> Result<Json<Alerts>, Problem> {
    let config = state.monitoring.config();
    if !config.selectors.contains_key(&query.env) {
        return Err(malformed());
    }
    let url = configured(config.alertmanager.as_ref(), Source::Alertmanager)?;
    let filter = config.alerts_filter.as_deref();
    let filter = filter.ok_or(SourceError::NotConfigured(Source::Alertmanager))?;
    let source = AlertsSource {
        client: state.monitoring.client(),
        url,
    };
    let alerts = source.firing(&matchers(filter, &query.env)).await?;
    Ok(Json(Alerts { alerts }))
}

/// A Grafana Explore link to a request's logs and trace.
#[utoipa::path(
    get, path = "/links/grafana", tag = "monitoring", operation_id = "getGrafanaLink",
    params(GrafanaQuery),
    responses(
        (status = OK, description = "The link", body = GrafanaLink),
        (status = BAD_REQUEST, description = "`request.malformed`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = SERVICE_UNAVAILABLE, description = "`monitoring.not_configured`",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn grafana_link(
    State(state): State<AppState>,
    Query(query): Query<GrafanaQuery>,
) -> Result<Json<GrafanaLink>, Problem> {
    let range = range(&state, query.from.as_deref(), query.to.as_deref())?;
    if !is_request_id(&query.request_id) {
        return Err(malformed());
    }
    let monitoring = &state.monitoring;
    let selector = logs_selector(monitoring, &query.env)?;
    let grafana = configured(monitoring.config().grafana.as_ref(), Source::Grafana)?;
    let link = RequestLink {
        logs_selector: selector,
        request_id: &query.request_id,
        range,
    };
    let url = explore_url(grafana, &monitoring.config().grafana_datasources, &link);
    Ok(Json(GrafanaLink { url }))
}

/// The range of `from` and `to`, read at the accounts' clock.
fn range(state: &AppState, from: Option<&str>, to: Option<&str>) -> Result<TimeRange, Problem> {
    TimeRange::parse(from, to, state.admins.now()).ok_or_else(malformed)
}

/// The logs selector of `environment`.
fn logs_selector<'a>(monitoring: &'a Monitoring, environment: &str) -> Result<&'a str, Problem> {
    let selectors = monitoring.config().selectors.get(environment);
    let selectors = selectors.ok_or_else(malformed)?;
    let selector = selectors.logs.as_deref();
    Ok(selector.ok_or(SourceError::NotConfigured(Source::VictoriaLogs))?)
}

fn malformed() -> Problem {
    Problem::new(codes::REQUEST_MALFORMED)
}
