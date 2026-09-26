//! `mcp_tool_calls_total`, `mcp_tool_duration_seconds` and the export metrics. The only test of
//! this binary to install the recorder: the other tests count too, once it is.

use axum::body::Body;
use axum::http::{Method, Request};
use life_pixel_server::mcp::metrics;
use life_pixel_server::{app, telemetry};
use serde_json::json;

use crate::router::{empty, send};
use crate::stack::ApiStack;
use crate::tokens::{secret, tool_call, tool_output};
use crate::{animation, listed_titles};

/// Whether `text` has a series of `name` whose labels include each of `labels`.
fn has_series(text: &str, name: &str, labels: &[&str]) -> bool {
    text.lines().any(|line| {
        line.starts_with(&format!("{name}{{")) && labels.iter().all(|label| line.contains(label))
    })
}

/// Checks that `text` has a series of `name` whose labels include each of `labels`.
fn assert_series(text: &str, name: &str, labels: &[&str]) {
    assert!(
        has_series(text, name, labels),
        "{name} {labels:?} in {text}"
    );
}

/// Lists, creates without the scope to, and exports then downloads a GIF with a new token of a
/// new account.
async fn call_the_tools(stack: &ApiStack) {
    let ada = stack.sign_up("ada@example.org").await;
    let mascot = animation(stack, &ada, "Mascot").await;
    let secret = secret(stack, &ada, &["read", "export"]).await;
    listed_titles(stack, &secret).await;
    let create = json!({ "title": "Walk", "width": 16, "height": 16, "project_name": "Game" });
    stack
        .send(tool_call(&secret, ("create_animation", create)))
        .await;
    let arguments = json!({ "id": mascot["id"], "format": "gif" });
    let export = tool_output(&stack.send(tool_call(&secret, ("export", arguments))).await);
    let url = export["files"][0]["url"].as_str().unwrap();
    let path = url.strip_prefix("http://localhost:8460").unwrap();
    let download = Request::get(path).body(Body::empty()).unwrap();
    assert_eq!(stack.send(download).await.status.as_u16(), 200);
}

#[tokio::test]
async fn tool_calls_and_downloads_are_measured() {
    let handle = telemetry::recorder().unwrap_or_else(|error| panic!("recorder: {error}"));
    metrics::describe();
    call_the_tools(&ApiStack::new().await).await;

    let answer = send(app::metrics_router(handle), empty(Method::GET, "/metrics")).await;

    let text = String::from_utf8_lossy(&answer.body).into_owned();
    let calls = [
        ("list_animations", "ok"),
        ("create_animation", "error"),
        ("export", "ok"),
    ];
    for (tool, outcome) in calls {
        let tool = format!("tool=\"{tool}\"");
        let outcome = format!("outcome=\"{outcome}\"");
        assert_series(&text, "mcp_tool_calls_total", &[&tool, &outcome]);
        assert_series(&text, "mcp_tool_duration_seconds_count", &[&tool]);
    }
    let gif = "format=\"gif\"";
    assert_series(&text, "exports_total", &[gif, "outcome=\"ok\""]);
    assert_series(&text, "export_duration_seconds_count", &[gif]);
    assert_series(&text, "export_size_bytes_count", &[gif]);
}
