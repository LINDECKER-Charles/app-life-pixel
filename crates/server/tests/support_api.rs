//! The support routes over the local stack (H9): each route and code, the screenshot checked
//! before it is decoded and re-encoded without its metadata, the team's internal notes absent
//! from the person's view, and the sweeper's `support/` prefix. Behind the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "support_api/creation.rs"]
mod creation;
#[path = "common/mod.rs"]
mod router;
#[path = "support_api/screenshots.rs"]
mod screenshots;
#[path = "common/stack.rs"]
mod stack;
#[path = "support_api/threads.rs"]
mod threads;

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::request::Builder;
use axum::http::{Method, Request};
use serde_json::Value;

use crate::router::auth::Browser;
use crate::stack::{ApiStack, api};

/// The path of the support requests, under `/api/v1`.
pub const REQUESTS: &str = "/support-requests";
/// The context the app attaches.
pub const CONTEXT: &str =
    r#"{"appVersion":"0.1.0","platform":"web","language":"en","screen":"/editor/:animationId"}"#;
/// An id no row has.
pub const UNKNOWN: &str = "0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e";
/// The boundary between the parts of the forms sent.
const BOUNDARY: &str = "life-pixel-test-boundary";

/// A part of a form: text, or a file with its media type.
#[derive(Clone, Copy)]
pub enum Part<'a> {
    /// A text field.
    Text(&'a str),
    /// A file: its bytes and its media type.
    File(&'a [u8], &'a str),
}

/// The `multipart/form-data` body of `parts`.
pub fn form_body(parts: &[(&str, Part<'_>)]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, part) in parts {
        body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        let disposition = format!("Content-Disposition: form-data; name=\"{name}\"");
        match part {
            Part::Text(text) => {
                body.extend_from_slice(format!("{disposition}\r\n\r\n").as_bytes());
                body.extend_from_slice(text.as_bytes());
            }
            Part::File(bytes, media_type) => {
                let headers = format!(
                    "{disposition}; filename=\"upload\"\r\nContent-Type: {media_type}\r\n\r\n"
                );
                body.extend_from_slice(headers.as_bytes());
                body.extend_from_slice(bytes);
            }
        }
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

/// The request of `builder`, with the form of `parts`.
pub fn with_form(builder: Builder, parts: &[(&str, Part<'_>)]) -> Request<Body> {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let builder = builder.header(CONTENT_TYPE, content_type);
    builder.body(Body::from(form_body(parts))).unwrap()
}

/// A new request of `browser` in `category`, saying `message`, with `screenshot` as a PNG.
pub fn new_request(
    browser: &Browser,
    (category, message): (&str, &str),
    screenshot: Option<&[u8]>,
) -> Request<Body> {
    let mut parts = vec![
        ("category", Part::Text(category)),
        ("message", Part::Text(message)),
        ("context", Part::Text(CONTEXT)),
    ];
    if let Some(bytes) = screenshot {
        parts.push(("screenshot", Part::File(bytes, "image/png")));
    }
    with_form(api(browser, Method::POST, REQUESTS), &parts)
}

/// Sends a bug report saying `message` for `browser`: the request answered.
pub async fn send_request(stack: &ApiStack, browser: &Browser, message: &str) -> Value {
    stack
        .created(new_request(browser, ("bug", message), None))
        .await
}

/// The id an answer's body carries.
pub fn id(body: &Value) -> String {
    body["id"].as_str().unwrap().to_owned()
}

/// Runs `statement` on the stack's database, with `id` as its `$1`.
pub async fn execute(stack: &ApiStack, statement: &'static str, id: &str) {
    let id = uuid::Uuid::parse_str(id).unwrap();
    let query = sqlx::query(statement).bind(id);
    query.execute(stack.database.pool()).await.unwrap();
}

/// The number of support requests stored.
pub async fn stored_requests(stack: &ApiStack) -> i64 {
    let query = sqlx::query_scalar("select count(*) from support_requests");
    query.fetch_one(stack.database.pool()).await.unwrap()
}
