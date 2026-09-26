//! Monitoring against fake VictoriaMetrics, VictoriaLogs and Alertmanager servers: the queries
//! the admin server writes, what it makes of the answers, and the problems of a source missing
//! or failing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use common::{NOTHING_LISTENS, get, json, send, signed_in};
use life_pixel_admin_server::monitoring;
use reqwest::Url;
use serde_json::{Value, json};

/// A query a fake source received: its path and its arguments.
#[derive(Clone, Debug)]
struct Asked {
    path: String,
    arguments: Vec<(String, String)>,
}

impl Asked {
    fn argument(&self, name: &str) -> Option<&str> {
        let found = self.arguments.iter().find(|(key, _)| key == name);
        found.map(|(_, value)| value.as_str())
    }

    fn all(&self, name: &str) -> Vec<&str> {
        let found = self.arguments.iter().filter(|(key, _)| key == name);
        found.map(|(_, value)| value.as_str()).collect()
    }
}

type Log = Arc<Mutex<Vec<Asked>>>;

async fn fake_sources(State(log): State<Log>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let url = Url::parse(&format!("http://fake{}", request.uri())).unwrap();
    let arguments: Vec<(String, String)> = url.query_pairs().into_owned().collect();
    let asked = Asked { path, arguments };
    log.lock().unwrap().push(asked.clone());
    answer(&asked)
}

fn answer(asked: &Asked) -> Response {
    match asked.path.as_str() {
        "/vm/api/v1/query" => instant(asked),
        "/vm/api/v1/query_range" => vector(json!([{
            "metric": { "route": "/api/v1/projects" },
            "values": [[1000, "1.5"], [1060, "+Inf"]],
        }])),
        "/vl/select/logsql/query" => concat!(
            r#"{"_time":"2027-01-15T08:00:00Z","_msg":"older","_stream":"{}","level":"INFO"}"#,
            "\n",
            r#"{"_time":"2027-01-15T09:00:00Z","_msg":"newer","request_id":"r1","status":"500"}"#,
            "\n",
        )
        .into_response(),
        "/am/api/v2/alerts" => axum::Json(json!([
            alert("a", "2027-01-15T08:00:00Z"),
            alert("b", "2027-01-15T09:00:00Z"),
        ]))
        .into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Two samples: the errors' first is `NaN`.
fn instant(asked: &Asked) -> Response {
    let value = if asked.argument("query").unwrap().contains("5xx") {
        "NaN"
    } else {
        "2"
    };
    vector(json!([
        { "metric": {}, "value": [1, value] },
        { "metric": {}, "value": [1, "3"] },
    ]))
}

fn vector(result: Value) -> Response {
    axum::Json(json!({ "status": "success", "data": { "resultType": "vector", "result": result } }))
        .into_response()
}

fn alert(fingerprint: &str, starts_at: &str) -> Value {
    json!({
        "fingerprint": fingerprint,
        "labels": { "alertname": "HighErrorRate", "severity": "critical" },
        "annotations": { "summary": "5xx above 5%" },
        "startsAt": starts_at,
        "status": { "state": "active" },
    })
}

/// The monitoring routes, signed in, with every source at `sources`.
fn monitoring_at(sources: Option<&str>) -> Router {
    let env = common::local_env(NOTHING_LISTENS, sources);
    signed_in(monitoring::routes::router().into(), common::state(&env))
}

/// The monitoring routes, with every source a fake one recording what it is asked.
async fn monitoring_with_fakes() -> (Router, Log) {
    let log = Log::default();
    let fake = Router::new()
        .fallback(fake_sources)
        .with_state(Arc::clone(&log));
    let base = common::spawn(fake).await;
    (monitoring_at(Some(&base)), log)
}

fn asked(log: &Log, path: &str) -> Vec<Asked> {
    let log = log.lock().unwrap();
    log.iter()
        .filter(|asked| asked.path == path)
        .cloned()
        .collect()
}

#[tokio::test]
async fn the_overview_asks_each_tile_now_and_one_range_earlier() {
    let (router, log) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/monitoring/overview?env=staging&from=1800000000&to=1800003600";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let overview = json(&body);
    let panels: Vec<&str> = overview["tiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tile| tile["panel"].as_str().unwrap())
        .collect();
    assert_eq!(
        panels,
        [
            "up", "errors", "latency", "requests", "exports", "mcp", "storage"
        ]
    );
    assert_eq!(overview["tiles"][0]["value"], 5.0);
    assert_eq!(
        overview["tiles"][1]["value"], 3.0,
        "NaN is left out of the sum"
    );
    assert_tile_queries(&asked(&log, "/vm/api/v1/query"));
}

/// Each tile is asked at both ends of the range, over the range.
fn assert_tile_queries(queries: &[Asked]) {
    assert_eq!(queries.len(), 14);
    let up: Vec<&Asked> = queries
        .iter()
        .filter(|asked| asked.argument("query") == Some("min(up{environment=\"staging\"})"))
        .collect();
    let mut times: Vec<&str> = up
        .iter()
        .map(|asked| asked.argument("time").unwrap())
        .collect();
    times.sort_unstable();
    assert_eq!(times, ["1800000000", "1800003600"]);
    let exports = queries.iter().find_map(|asked| {
        let query = asked.argument("query")?;
        query.contains("exports_total").then_some(query)
    });
    let expected = "sum by (format, outcome) \
                    (increase(exports_total{environment=\"staging\"}[3600s]))";
    assert_eq!(exports, Some(expected));
}

#[tokio::test]
async fn a_series_lays_the_previous_period_over_the_current_one() {
    let (router, log) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/monitoring/series?env=staging&panel=latency&from=1800000000&to=1800003600&step=60";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let series = json(&body);
    assert_eq!(series["stepSeconds"], 60);
    let names: Vec<&str> = series["series"]
        .as_array()
        .unwrap()
        .iter()
        .map(|series| series["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["p95", "p50", "p99"]);
    assert_eq!(series["series"][0]["values"], json!([1.5, null]));
    assert_eq!(series["previous"][0]["times"], json!([4600.0, 4660.0]));
    let ranges = asked(&log, "/vm/api/v1/query_range");
    assert_eq!(ranges.len(), 6);
    let previous = ranges
        .iter()
        .filter(|asked| asked.argument("end") == Some("1800000000"));
    assert_eq!(previous.count(), 3);
    assert!(
        ranges
            .iter()
            .all(|asked| asked.argument("step") == Some("60s"))
    );
}

#[tokio::test]
async fn container_panels_read_the_containers_selector() {
    let (router, log) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/monitoring/series?env=staging&panel=memory";
    let (status, _, _) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    let query = asked(&log, "/vm/api/v1/query_range")[0]
        .argument("query")
        .unwrap()
        .to_owned();
    let expected = "container_memory_working_set_bytes\
                    {container_label_com_docker_compose_project=\"life-pixel-staging\"}";
    assert_eq!(query, expected);
}

#[tokio::test]
async fn a_log_search_escapes_its_text_and_caps_its_lines() {
    let (router, log) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/logs?env=staging&level=error&q=say%20%22hi%22%20OR%20*&limit=9999";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let lines = json(&body);
    assert_eq!(lines["lines"][0]["message"], "newer");
    assert_eq!(lines["lines"][0]["requestId"], "r1");
    assert_eq!(lines["lines"][1]["level"], "INFO");
    let query = &asked(&log, "/vl/select/logsql/query")[0];
    let expected =
        "{compose_project=\"life-pixel-staging\"} AND level:i(error) AND \"say \\\"hi\\\" OR *\"";
    assert_eq!(query.argument("query"), Some(expected));
    assert_eq!(query.argument("limit"), Some("500"));
}

#[tokio::test]
async fn alerts_are_filtered_for_the_environment_newest_first() {
    let (router, log) = monitoring_with_fakes().await;
    let (status, _, body) = send(&router, get("/api/admin/v1/alerts?env=staging")).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let alerts = json(&body);
    assert_eq!(alerts["alerts"][0]["fingerprint"], "b");
    assert_eq!(alerts["alerts"][0]["name"], "HighErrorRate");
    assert_eq!(alerts["alerts"][0]["summary"], "5xx above 5%");
    let asked = &asked(&log, "/am/api/v2/alerts")[0];
    assert_eq!(
        asked.all("filter"),
        ["project=\"life-pixel\"", "environment=\"staging\""]
    );
    assert_eq!(asked.argument("active"), Some("true"));
    assert_eq!(asked.argument("silenced"), Some("false"));
}

#[tokio::test]
async fn a_grafana_link_opens_the_request_in_explore() {
    let (router, _) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/links/grafana?env=staging&requestId=0190-abc";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::OK);
    let url = json(&body)["url"].as_str().unwrap().to_owned();
    let parsed = Url::parse(&url).unwrap();
    assert!(parsed.path().ends_with("/grafana/explore"), "{url}");
    let panes: HashMap<String, String> = parsed.query_pairs().into_owned().collect();
    let panes: Value = serde_json::from_str(&panes["panes"]).unwrap();
    assert_eq!(panes["logs"]["datasource"], "vl");
    let bad = "/api/admin/v1/links/grafana?env=staging&requestId=a%20b";
    assert_eq!(send(&router, get(bad)).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_source_left_empty_is_not_configured() {
    let router = monitoring_at(None);
    for (path, source) in [
        (
            "/api/admin/v1/monitoring/overview?env=staging",
            "victoriametrics",
        ),
        ("/api/admin/v1/logs?env=staging", "victorialogs"),
        ("/api/admin/v1/alerts?env=staging", "alertmanager"),
        (
            "/api/admin/v1/links/grafana?env=staging&requestId=r1",
            "grafana",
        ),
    ] {
        let (status, _, body) = send(&router, get(path)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{path}");
        let problem = json(&body);
        assert_eq!(problem["code"], "monitoring.not_configured");
        assert_eq!(problem["params"]["source"], source);
    }
}

#[tokio::test]
async fn an_environment_without_selectors_is_not_configured() {
    let (router, _) = monitoring_with_fakes().await;
    let path = "/api/admin/v1/monitoring/overview?env=production";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json(&body)["code"], "monitoring.not_configured");
}

#[tokio::test]
async fn a_failing_source_is_unavailable() {
    let router = monitoring_at(Some(NOTHING_LISTENS));
    let path = "/api/admin/v1/logs?env=staging";
    let (status, _, body) = send(&router, get(path)).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let problem = json(&body);
    assert_eq!(problem["code"], "monitoring.unavailable");
    assert_eq!(problem["params"]["source"], "victorialogs");
}

#[tokio::test]
async fn an_unknown_environment_panel_or_time_is_malformed() {
    let (router, log) = monitoring_with_fakes().await;
    for path in [
        "/api/admin/v1/monitoring/overview?env=moon",
        "/api/admin/v1/monitoring/overview?env=staging&from=yesterday",
        "/api/admin/v1/monitoring/overview?env=staging&from=now-100d",
        "/api/admin/v1/monitoring/series?env=staging&panel=sql",
        "/api/admin/v1/alerts?env=moon",
    ] {
        let (status, _, body) = send(&router, get(path)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(json(&body)["code"], "request.malformed", "{path}");
    }
    assert!(log.lock().unwrap().is_empty());
}
