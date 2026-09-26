//! The storage quota: a write beyond it is refused and counted in `quota_rejections_total`;
//! a deletion always goes through.

use axum::http::Method;
use life_pixel_server::routes::library::metrics;
use life_pixel_server::{app, telemetry};
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;

use crate::router::{empty, send};
use crate::stack::{ApiStack, delete, import, sample};
use crate::{id, json_call};

/// The series of the storage quota's refusals.
const REJECTIONS: &str = "quota_rejections_total{kind=\"storage\"}";

/// A stack whose free plan stores `limit` bytes.
async fn stack_storing(limit: usize) -> ApiStack {
    let limit = limit.to_string();
    ApiStack::with(|env| {
        env.insert("LP_PLAN_FREE_STORAGE_BYTES".to_owned(), limit);
    })
    .await
}

/// The lines of the metrics that `handle` renders.
async fn metrics_text(handle: PrometheusHandle) -> String {
    let answer = send(app::metrics_router(handle), empty(Method::GET, "/metrics")).await;
    String::from_utf8_lossy(&answer.body).into_owned()
}

#[tokio::test]
async fn the_quota_refuses_writes_and_counts_them_but_never_deletions() {
    let handle = telemetry::recorder().unwrap_or_else(|error| panic!("recorder: {error}"));
    metrics::describe();
    let limit = sample("Walk").len() * 3 / 2;
    let stack = stack_storing(limit).await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let walk = stack.created(import(&ada, &sprites, &sample("Walk"))).await;
    let (used, walk) = (walk["documentBytes"].clone(), id(&walk));
    let exceeded = (409, "quota.storage_exceeded");

    let again = import(&ada, &sprites, &sample("Walk"));
    let params = stack.refused(again, exceeded).await;
    let path = format!("/animations/{walk}/duplicate");
    let copy = json_call(&ada, (Method::POST, &path), &json!({ "title": "Twice" }));
    stack.refused(copy, exceeded).await;
    stack
        .expect(204, delete(&ada, &format!("/animations/{walk}")))
        .await;
    stack.created(import(&ada, &sprites, &sample("Walk"))).await;

    let expected = json!({ "used": used, "limit": limit, "requested": used });
    assert_eq!(params, expected);
    let text = metrics_text(handle).await;
    let series = format!("{REJECTIONS} 2");
    assert!(
        text.lines().any(|line| line == series),
        "{series} in {text}"
    );
}
