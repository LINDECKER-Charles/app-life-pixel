//! What the token, MCP and admin tests share (A3): creating a personal access token, and
//! calling the MCP endpoint with it.

#![allow(dead_code)] // Each test file uses its own share of the helpers.

use axum::body::Body;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST};
use axum::http::{Method, Request};
use serde_json::{Value, json};

use crate::router::Answer;
use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, api};

/// The endpoint's path.
pub const MCP: &str = "/mcp";
/// The authority clients reach the endpoint at: `LP_PUBLIC_URL`'s.
const AUTHORITY: &str = "localhost:8460";
/// What a Streamable HTTP client accepts.
const MCP_ACCEPT: &str = "application/json, text/event-stream";

/// Creates a token of `browser` named `name`, granting `scopes`, for `days`: the answer.
pub async fn create_token(
    stack: &ApiStack,
    browser: &Browser,
    (name, scopes, days): (&str, &[&str], u16),
) -> Value {
    let body = json!({ "name": name, "scopes": scopes, "expiresInDays": days });
    stack
        .created(with_json(api(browser, Method::POST, "/tokens"), &body))
        .await
}

/// The secret of a new token of `browser` granting `scopes` for 90 days.
pub async fn secret(stack: &ApiStack, browser: &Browser, scopes: &[&str]) -> String {
    let created = create_token(stack, browser, ("Claude Code", scopes, 90)).await;
    created["token"].as_str().unwrap().to_owned()
}

/// The JSON-RPC request `method` with `params`, as a client posts it to [`MCP`], with
/// `authorization` when given.
pub fn rpc(authorization: Option<&str>, (method, params): (&str, Value)) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(MCP)
        .header(HOST, AUTHORITY)
        .header(ACCEPT, MCP_ACCEPT)
        .header(CONTENT_TYPE, "application/json");
    if let Some(authorization) = authorization {
        builder = builder.header(AUTHORIZATION, authorization);
    }
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    builder.body(Body::from(body.to_string())).unwrap()
}

/// `tools/list`, with the token `secret`.
pub fn list_tools(secret: &str) -> Request<Body> {
    rpc(Some(&format!("Bearer {secret}")), ("tools/list", json!({})))
}

/// A call of `tool` with `arguments`, with the token `secret`.
pub fn tool_call(secret: &str, (tool, arguments): (&str, Value)) -> Request<Body> {
    let params = json!({ "name": tool, "arguments": arguments });
    rpc(Some(&format!("Bearer {secret}")), ("tools/call", params))
}

/// The `result` of a JSON-RPC answer, checked to be `200`.
pub fn result(answer: &Answer) -> Value {
    assert_eq!(answer.status.as_u16(), 200, "{:?}", answer.body);
    let body = answer.json();
    assert!(body.get("error").is_none(), "{body}");
    body["result"].clone()
}

/// The structured result of a successful tool call.
pub fn tool_output(answer: &Answer) -> Value {
    let result = result(answer);
    assert_ne!(result["isError"], true, "{result}");
    result["structuredContent"].clone()
}

/// The `{code, params}` of a failed tool call.
pub fn tool_failure(answer: &Answer) -> Value {
    let result = result(answer);
    assert_eq!(result["isError"], true, "{result}");
    let text = result["content"][0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap()
}
