//! The whole router with a database that never answers: the origin and CSRF checks, the session
//! checks, the problems, the headers, the catalogues and `/healthz`. The flows that need a
//! database are the stack tests'.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::StatusCode;
use axum::http::header::{RETRY_AFTER, SET_COOKIE};
use common::{CONSOLE_ORIGIN, NOTHING_LISTENS, get, json, send};
use life_pixel_admin_server::app;

const FOREIGN_ORIGIN: &str = "https://attacker.example";
const SIGN_IN_BODY: &str =
    r#"{"email":"ops@example.org","password":"a long password","code":"123456"}"#;

fn router() -> Router {
    let env = common::local_env(NOTHING_LISTENS, None);
    let peer = SocketAddr::from(([203, 0, 113, 7], 50_000));
    app::router(common::state(&env)).layer(MockConnectInfo(peer))
}

fn post(path: &str, origin: Option<&str>, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::post(path).header("content-type", "application/json");
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder.body(Body::from(SIGN_IN_BODY)).unwrap()
}

fn code(body: &[u8]) -> String {
    json(body)["code"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn signing_in_from_another_origin_is_refused() {
    let router = router();
    for origin in [Some(FOREIGN_ORIGIN), None] {
        let request = post("/api/admin/v1/auth/sign-in", origin, None);
        let (status, _, body) = send(&router, request).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{origin:?}");
        assert_eq!(code(&body), "admin.csrf");
    }
}

#[tokio::test]
async fn signing_in_is_limited_to_five_attempts_a_minute() {
    let router = router();
    for _ in 0..5 {
        let request = post("/api/admin/v1/auth/sign-in", Some(CONSOLE_ORIGIN), None);
        let (status, _, _) = send(&router, request).await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "the database never answers"
        );
    }
    let request = post("/api/admin/v1/auth/sign-in", Some(CONSOLE_ORIGIN), None);
    let (status, headers, body) = send(&router, request).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(&body), "rate_limit.exceeded");
    assert!(headers.get(RETRY_AFTER).is_some());
}

#[tokio::test]
async fn a_change_with_the_cookie_from_another_origin_is_refused() {
    let router = router();
    let cookie = Some("__Host-lpa_session=not-a-token");
    let request = post("/api/admin/v1/auth/sign-out", Some(FOREIGN_ORIGIN), cookie);
    let (status, _, body) = send(&router, request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(code(&body), "admin.csrf");
    let request = post(
        "/api/admin/v1/users/0190/suspend",
        Some(FOREIGN_ORIGIN),
        cookie,
    );
    assert_eq!(send(&router, request).await.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn signing_out_an_ended_session_clears_its_cookie() {
    let router = router();
    let cookie = Some("__Host-lpa_session=not-a-token");
    let request = post("/api/admin/v1/auth/sign-out", Some(CONSOLE_ORIGIN), cookie);
    let (status, headers, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let cleared = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cleared.starts_with("__Host-lpa_session=;"), "{cleared}");
    assert!(cleared.contains("Max-Age=0"), "{cleared}");
}

#[tokio::test]
async fn every_admin_route_needs_a_live_session() {
    let router = router();
    for path in [
        "/api/admin/v1/auth/session",
        "/api/admin/v1/monitoring/overview?env=staging",
        "/api/admin/v1/logs?env=staging",
        "/api/admin/v1/users",
        "/api/admin/v1/support-requests/0190",
    ] {
        let (status, _, body) = send(&router, get(path)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(code(&body), "admin.unauthenticated");
    }
    let request = Request::get("/api/admin/v1/auth/session")
        .header("cookie", "__Host-lpa_session=not-a-token")
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        headers.get(SET_COOKIE).is_some(),
        "an ended session's cookie is cleared"
    );
}

#[tokio::test]
async fn errors_are_problems_and_answers_carry_the_security_headers() {
    let router = router();
    let (status, headers, body) = send(&router, get("/api/admin/v1/nothing-here")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        headers.get("content-type").unwrap(),
        "application/problem+json"
    );
    assert_eq!(code(&body), "request.not_found");
    assert_eq!(headers.get("x-robots-tag").unwrap(), "noindex, nofollow");
    assert!(headers.get("content-security-policy").is_some());
    assert!(headers.get("x-request-id").is_some());
    let request = Request::post("/api/admin/v1/auth/sign-in")
        .header("origin", CONSOLE_ORIGIN)
        .header("content-type", "application/json")
        .body(Body::from(vec![b'x'; 128 * 1024]))
        .unwrap();
    let (status, _, body) = send(&router, request).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(json(&body)["params"]["maxBytes"], 64 * 1024);
}

#[tokio::test]
async fn the_catalogues_are_served_and_health_follows_the_database() {
    let router = router();
    let (status, _, body) = send(&router, get("/i18n/en.json")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json(&body)["errors.admin.invalid_credentials"].is_string());
    assert_eq!(
        send(&router, get("/i18n/xx.json")).await.0,
        StatusCode::NOT_FOUND
    );
    let (status, _, body) = send(&router, get("/healthz")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(&body), "service.unavailable");
}
