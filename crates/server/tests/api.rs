//! `/api/v1`: the client-version check, problems for unknown routes and methods, the body limit
//! and the `api` rate limit.

mod common;

use std::net::SocketAddr;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::Method;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, post};
use common::{TestServer, client_peer, empty, get_with, request, send};
use life_pixel_server::http::problem::{BodyLimit, ensure_problem};
use life_pixel_server::http::rate_limit::limit_api;
use serde_json::json;

fn with_client(client: &str) -> axum::http::Request<Body> {
    get_with("/api/v1/nothing", "life-pixel-client", client)
}

fn minimum_web_1_2() -> TestServer {
    TestServer::with(
        |env| {
            let minimums = "web=1.2.0,desktop=1.0.0".to_owned();
            env.insert("LP_MIN_CLIENT_VERSIONS".to_owned(), minimums);
        },
        true,
    )
}

#[tokio::test]
async fn an_outdated_client_must_update() {
    let answer = minimum_web_1_2().send(with_client("web/1.1.9")).await;
    let params = answer.assert_problem(426, "client.update_required");
    assert_eq!(params, json!({ "minimum": "1.2.0" }));
}

#[tokio::test]
async fn a_current_client_a_script_and_an_unknown_platform_pass() {
    let server = minimum_web_1_2();
    for client in ["web/1.2.0", "desktop/1.0.0", "cli/0.1.0"] {
        let answer = server.send(with_client(client)).await;
        answer.assert_problem(404, "request.not_found");
    }
    server
        .get("/api/v1/nothing")
        .await
        .assert_problem(404, "request.not_found");
}

#[tokio::test]
async fn a_malformed_client_header_is_refused() {
    for client in ["web", "web/one", "/1.0.0"] {
        let answer = minimum_web_1_2().send(with_client(client)).await;
        answer.assert_problem(400, "request.malformed");
    }
}

#[tokio::test]
async fn an_unknown_api_route_is_a_problem_never_the_app() {
    let server = TestServer::new();
    for path in [
        "/api/v1",
        "/api/v1/",
        "/api/v1/unknown/42",
        "/api/v2/projects",
    ] {
        server
            .get(path)
            .await
            .assert_problem(404, "request.not_found");
    }
    let post = empty(Method::POST, "/api/v1/unknown");
    server
        .send(post)
        .await
        .assert_problem(404, "request.not_found");
}

/// A route reading its body, behind the API's layers.
fn echo_router(max_bytes: usize) -> Router {
    Router::new()
        .route("/echo", post(|body: Bytes| async move { body }))
        .body_limit(max_bytes)
        .layer(from_fn(ensure_problem))
}

#[tokio::test]
async fn a_method_the_route_does_not_accept_is_a_problem_with_allow() {
    let answer = send(echo_router(16), empty(Method::GET, "/echo")).await;
    answer.assert_problem(405, "request.method_not_allowed");
    assert_eq!(answer.header("allow"), "POST");
}

#[tokio::test]
async fn a_body_above_the_limit_is_too_large() {
    let within = request(Method::POST, "/echo")
        .body(Body::from("0123456789abcdef"))
        .unwrap();
    assert_eq!(send(echo_router(16), within).await.body, "0123456789abcdef");
    let above = request(Method::POST, "/echo")
        .body(Body::from("0123456789abcdefg"))
        .unwrap();
    let params = send(echo_router(16), above)
        .await
        .assert_problem(413, "request.too_large");
    assert_eq!(params, json!({ "maxBytes": 16 }));
}

#[tokio::test]
async fn the_api_limit_answers_429_with_retry_after_per_address() {
    let server = TestServer::new();
    let limited = |peer: SocketAddr| {
        Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(from_fn_with_state(server.state.clone(), limit_api))
            .with_state(server.state.clone())
            .layer(axum::extract::connect_info::MockConnectInfo(peer))
    };
    let ping = || empty(Method::GET, "/ping");
    for _ in 0..600 {
        assert_eq!(send(limited(client_peer()), ping()).await.body, "pong");
    }
    let refused = send(limited(client_peer()), ping()).await;
    let params = refused.assert_problem(429, "rate_limit.exceeded");
    assert_eq!(params, json!({ "retryAfterSeconds": 1 }));
    assert_eq!(refused.header("retry-after"), "1");
    let other = SocketAddr::from(([192, 0, 2, 11], 50_000));
    assert_eq!(send(limited(other), ping()).await.body, "pong");
}
