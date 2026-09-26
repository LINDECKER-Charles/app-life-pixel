//! The hosted MCP endpoint over the local stack (A3): the bearer token and its failures, the
//! scopes, the `mcp` rate limit, the daily ceiling and its reset at midnight UTC, the signed
//! links of `export`, and the metrics. Behind the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "mcp_api/access.rs"]
mod access;
#[path = "mcp_api/ceiling.rs"]
mod ceiling;
#[path = "mcp_api/links.rs"]
mod links;
#[path = "mcp_api/metrics.rs"]
mod metrics;
#[path = "common/mod.rs"]
mod router;
#[path = "common/stack.rs"]
mod stack;
#[path = "common/tokens.rs"]
mod tokens;

use std::sync::Arc;

use life_pixel_service::memory::FixedClock;
use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::router::auth::Browser;
use crate::stack::{ApiStack, import, sample};
use crate::tokens::{tool_call, tool_output};

/// The endpoint's server over a new database whose clock is `clock`, and its configuration
/// adjusted by `change`.
pub async fn stack_at(
    clock: &Arc<FixedClock>,
    change: impl FnOnce(&mut std::collections::HashMap<String, String>),
) -> ApiStack {
    ApiStack::with_clock(change, clock.clone()).await
}

/// A clock at `at`.
pub fn clock_at(at: OffsetDateTime) -> Arc<FixedClock> {
    Arc::new(FixedClock::new(at))
}

/// Imports the sample animation `title` into a new project of `browser`: the animation.
pub async fn animation(stack: &ApiStack, browser: &Browser, title: &str) -> Value {
    let project = stack.project(browser, "Game").await;
    stack
        .created(import(browser, &project, &sample(title)))
        .await
}

/// The titles `list_animations` answers to the token `secret`.
pub async fn listed_titles(stack: &ApiStack, secret: &str) -> Vec<String> {
    let answer = stack
        .send(tool_call(secret, ("list_animations", json!({}))))
        .await;
    let listed = tool_output(&answer);
    let animations = listed["animations"].as_array().unwrap();
    animations
        .iter()
        .map(|animation| animation["title"].as_str().unwrap().to_owned())
        .collect()
}
