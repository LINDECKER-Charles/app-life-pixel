//! `/metrics`: the `http_*` metrics, by route template, never by raw path. Its own test binary,
//! as the recorder is the process's.

mod common;

use axum::http::Method;
use common::{TestServer, empty, send};
use life_pixel_server::app;
use life_pixel_server::telemetry;

/// A route, an i18n file and a missing one, an app page and an unknown API route.
const PATHS: [&str; 5] = [
    "/healthz",
    "/i18n/en.json",
    "/i18n/de.json",
    "/editor/secret-7",
    "/api/v1/users/ada",
];

/// The text of `/metrics` once [`PATHS`] are requested.
async fn metrics_after_requests() -> String {
    let handle = telemetry::recorder().unwrap_or_else(|error| panic!("recorder: {error}"));
    let server = TestServer::new();
    for path in PATHS {
        server.get(path).await;
    }
    let answer = send(app::metrics_router(handle), empty(Method::GET, "/metrics")).await;
    assert_eq!(answer.status, 200);
    assert!(answer.header("content-type").starts_with("text/plain"));
    String::from_utf8_lossy(&answer.body).into_owned()
}

/// Whether `text` counts a request of `route` answered with `class`.
fn counts(text: &str, route: &str, class: &str) -> bool {
    let route = format!("route=\"{route}\"");
    let class = format!("status_class=\"{class}\"");
    text.lines().any(|line| {
        line.starts_with("http_requests_total{") && line.contains(&route) && line.contains(&class)
    })
}

#[tokio::test]
async fn requests_are_counted_by_route_template_never_by_path() {
    let text = metrics_after_requests().await;
    assert!(counts(&text, "/healthz", "2xx"), "{text}");
    assert!(counts(&text, "/i18n/{file}", "2xx"), "{text}");
    assert!(counts(&text, "/i18n/{file}", "4xx"), "{text}");
    assert!(counts(&text, "unmatched", "2xx"), "{text}");
    assert!(counts(&text, "unmatched", "4xx"), "{text}");
    for raw in ["secret-7", "ada", "de.json", "en.json"] {
        assert!(!text.contains(raw), "{raw} in {text}");
    }
    let durations = "http_request_duration_seconds_bucket{route=\"/healthz\",method=\"GET\"";
    assert!(text.contains(durations), "{text}");
}
