//! The library routes over the local stack (H6): each route and its problems, the version of a
//! document under `If-Match`, the storage quota and `quota_rejections_total`, and one account
//! seeing nothing of another's. Behind the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "library_api/crud.rs"]
mod crud;
#[path = "library_api/isolation.rs"]
mod isolation;
#[path = "library_api/quota.rs"]
mod quota;
#[path = "library_api/refusals.rs"]
mod refusals;
#[path = "common/mod.rs"]
mod router;
#[path = "common/stack.rs"]
mod stack;
#[path = "library_api/versions.rs"]
mod versions;

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::Value;

use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, api, get};

/// An id no row has.
pub const UNKNOWN: &str = "0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e";

/// `method path` with `body` as JSON, for `browser`.
pub fn json_call(browser: &Browser, (method, path): (Method, &str), body: &Value) -> Request<Body> {
    with_json(api(browser, method, path), body)
}

/// The id an answer's body carries.
pub fn id(body: &Value) -> String {
    body["id"].as_str().unwrap().to_owned()
}

/// The titles of `GET /animations<query>`.
pub async fn titles(stack: &ApiStack, browser: &Browser, query: &str) -> Vec<String> {
    let answer = stack
        .expect(200, get(browser, &format!("/animations{query}")))
        .await;
    let items = answer.json()["items"].as_array().unwrap().clone();
    items
        .iter()
        .map(|item| item["title"].as_str().unwrap().to_owned())
        .collect()
}
