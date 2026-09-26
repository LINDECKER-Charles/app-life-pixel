//! `docs/v1/mcp-cli.md`'s "A5 — MCP end to end", over the hosted HTTP transport (`stack-tests`):
//! the scripted session played by a real `rmcp` Streamable HTTP client against the server bound
//! on a free port with a test database and storage, its `export` downloaded through its signed
//! links and run in `wasmi`; a missing scope and the daily ceiling.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

#[path = "common/mod.rs"]
mod router;
#[path = "../../cli/tests/session.rs"]
mod session;
#[path = "common/stack.rs"]
mod stack;
#[path = "common/tokens.rs"]
mod tokens;

use std::net::{Ipv4Addr, SocketAddr};

use axum::http::Method;
use axum::http::header::ORIGIN;
use rmcp::service::RunningService;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::router::auth::{Browser, DEV_ORIGIN, PASSWORD, with_json};
use crate::router::request;
use crate::stack::{API, ApiStack};
use crate::tokens::{tool_call, tool_failure};

/// The server's public router, bound on a free port of `127.0.0.1`, until dropped.
struct RunningServer {
    base_url: String,
    handle: JoinHandle<()>,
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// A test stack whose `LP_PUBLIC_URL` and `LP_HTTP_ADDR` already name the free port its server
/// binds: `export`'s signed links point back at it.
async fn stack_with_server() -> (ApiStack, RunningServer) {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");
    let public_url = base_url.clone();
    let stack = ApiStack::with(move |env| {
        env.insert("LP_PUBLIC_URL".to_owned(), public_url);
        env.insert("LP_HTTP_ADDR".to_owned(), format!("127.0.0.1:{port}"));
    })
    .await;
    let router = life_pixel_server::app::public_router(stack.state.clone());
    let service = router.into_make_service_with_connect_info::<SocketAddr>();
    let handle = tokio::spawn(async move {
        let _stopped = axum::serve(listener, service).await;
    });
    (stack, RunningServer { base_url, handle })
}

/// An `rmcp` Streamable HTTP client, authenticated with `token`, connected to `server`'s `/mcp`.
async fn connect(server: &RunningServer, token: &str) -> RunningService<RoleClient, ()> {
    let config = StreamableHttpClientTransportConfig::with_uri(format!("{}/mcp", server.base_url))
        .auth_header(token.to_owned());
    let transport = StreamableHttpClientTransport::from_config(config);
    ().serve(transport).await.unwrap()
}

/// `stack.sign_up` and `tokens::secret` send their requests from `LP_PUBLIC_URL`'s default
/// origin, which [`stack_with_server`] moves away from its default to the free port it binds; so
/// a stack of a real server signs up and mints tokens from [`DEV_ORIGIN`] instead, one of
/// `LP_ALLOWED_ORIGINS`, which `stack_with_server` leaves untouched.
async fn sign_up_over(stack: &ApiStack, email: &str) -> Browser {
    let body = json!({ "email": email, "password": PASSWORD, "language": "en" });
    let request = with_json(from_dev(Method::POST, "/api/v1/auth/sign-up"), &body);
    let answer = stack.send(request).await;
    assert_eq!(answer.status, 201, "{:?}", answer.body);
    Browser::of(&answer)
}

/// The secret of a new token of `browser` granting `scopes` for 90 days, from [`DEV_ORIGIN`].
async fn secret_over(stack: &ApiStack, browser: &Browser, scopes: &[&str]) -> String {
    let body = json!({ "name": "Session", "scopes": scopes, "expiresInDays": 90 });
    let path = format!("{API}/tokens");
    let request = with_json(browser.signed(from_dev(Method::POST, &path)), &body);
    let created = stack.created(request).await;
    created["token"].as_str().unwrap().to_owned()
}

/// A request builder for `method` and `path`, from [`DEV_ORIGIN`].
fn from_dev(method: Method, path: &str) -> axum::http::request::Builder {
    request(method, path).header(ORIGIN, DEV_ORIGIN)
}

#[tokio::test]
async fn the_scripted_session_plays_over_http_and_its_wasm_export_runs_in_wasmi() {
    let (stack, server) = stack_with_server().await;
    let ada = sign_up_over(&stack, "ada@example.org").await;
    let token = secret_over(&stack, &ada, &["read", "write", "export"]).await;
    let client = connect(&server, &token).await;

    let played = session::play(&client).await;

    let export_arguments = json!({ "id": played.animation_id, "format": "wasm" });
    let export = session::call(&client, "export", export_arguments).await;
    let files = export["files"].as_array().unwrap();
    let wasm_file = files
        .iter()
        .find(|file| file["name"].as_str().unwrap().ends_with(".wasm"))
        .unwrap_or_else(|| panic!("a .wasm file among {files:?}"));
    let url = wasm_file["url"].as_str().unwrap();
    let response = reqwest::get(url).await.unwrap();
    assert_eq!(response.status().as_u16(), 200, "{url}");
    let bytes = response.bytes().await.unwrap().to_vec();
    session::check_wasm(&bytes, &played.frame0_rows);

    session::check_embed_snippet(&client, &played.animation_id).await;

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn a_token_missing_the_write_scope_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let reader = crate::tokens::secret(&stack, &ada, &["read"]).await;
    let create = json!({ "title": "Walk", "width": 16, "height": 16, "project_name": "Game" });

    let failure = tool_failure(
        &stack
            .send(tool_call(&reader, ("create_animation", create)))
            .await,
    );

    let expected = json!({ "code": "token.scope", "params": { "required": "write" } });
    assert_eq!(failure, expected);
}

#[tokio::test]
async fn the_daily_ceiling_refuses_calls_beyond_the_plan() {
    let one_a_day = |env: &mut std::collections::HashMap<String, String>| {
        env.insert("LP_PLAN_FREE_MCP_CALLS_PER_DAY".to_owned(), "1".to_owned());
    };
    let stack = ApiStack::with(one_a_day).await;
    let ada = stack.sign_up("ada@example.org").await;
    let reader = crate::tokens::secret(&stack, &ada, &["read"]).await;
    let list = || tool_call(&reader, ("list_animations", json!({})));

    let first = stack.send(list()).await;
    assert_eq!(first.status.as_u16(), 200, "{:?}", first.body);
    let refused = tool_failure(&stack.send(list()).await);

    assert_eq!(refused["code"], "mcp.daily_limit");
    assert_eq!(refused["params"]["limit"], 1);
}
