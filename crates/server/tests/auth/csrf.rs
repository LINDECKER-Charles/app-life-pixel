//! CSRF: an unsafe request carrying the session cookie comes from an allowed origin with its
//! session's `X-CSRF-Token`; signing up and in check the origin alone.

use axum::http::Method;
use axum::http::header::ORIGIN;
use serde_json::json;

use crate::common::auth::{
    Browser, DEV_ORIGIN, NEW_PASSWORD, OTHER_ORIGIN, PASSWORD, from_app, sign_in, sign_up,
    with_json,
};
use crate::common::{TestServer, request};

const PASSWORD_PATH: &str = "/api/v1/auth/password";

/// A server with one account and two of its sessions.
async fn two_sessions() -> (TestServer, Browser, Browser) {
    let server = TestServer::new();
    let first = Browser::of(&server.send(sign_up("ada@example.org", "en")).await);
    let second = Browser::of(&server.send(sign_in("ada@example.org", PASSWORD)).await);
    (server, first, second)
}

/// A change of password.
fn change() -> serde_json::Value {
    json!({ "currentPassword": PASSWORD, "newPassword": NEW_PASSWORD })
}

#[tokio::test]
async fn an_unsafe_request_without_its_sessions_token_is_refused() {
    let (server, first, second) = two_sessions().await;
    let without = with_json(
        first.cookie(from_app(Method::PUT, PASSWORD_PATH)),
        &change(),
    );
    server.send(without).await.assert_problem(403, "auth.csrf");
    let borrowed = Browser {
        cookie: first.cookie.clone(),
        csrf: second.csrf.clone(),
    };
    let wrong = server
        .send(borrowed.change_password(PASSWORD, NEW_PASSWORD))
        .await;
    wrong.assert_problem(403, "auth.csrf");
    let garbled = Browser {
        cookie: first.cookie.clone(),
        csrf: "not base64url!".to_owned(),
    };
    let garbled = server
        .send(garbled.change_password(PASSWORD, NEW_PASSWORD))
        .await;
    garbled.assert_problem(403, "auth.csrf");
    let sign_in_again = server.send(sign_in("ada@example.org", PASSWORD)).await;
    assert_eq!(sign_in_again.status, 200, "the password did not change");
}

#[tokio::test]
async fn an_unsafe_request_from_another_origin_is_refused_even_with_its_token() {
    let (server, first, _) = two_sessions().await;
    let elsewhere = request(Method::PUT, PASSWORD_PATH).header(ORIGIN, OTHER_ORIGIN);
    let answer = server
        .send(with_json(first.signed(elsewhere), &change()))
        .await;
    answer.assert_problem(403, "auth.csrf");
    let nowhere = request(Method::PUT, PASSWORD_PATH);
    let answer = server
        .send(with_json(first.signed(nowhere), &change()))
        .await;
    answer.assert_problem(403, "auth.csrf");
    let from_dev = request(Method::POST, "/api/v1/auth/sign-out").header(ORIGIN, DEV_ORIGIN);
    let answer = server
        .send(with_json(first.signed(from_dev), &json!({})))
        .await;
    assert_eq!(answer.status, 204, "an allowed origin");
}

#[tokio::test]
async fn a_safe_request_needs_no_token() {
    let (server, first, _) = two_sessions().await;
    let answer = server.send(first.get("/api/v1/auth/session")).await;
    assert_eq!(answer.status, 200);
}

#[tokio::test]
async fn opening_a_session_from_another_origin_is_refused() {
    let server = TestServer::new();
    let sign_up = json!({ "email": "ada@example.org", "password": PASSWORD, "language": "en" });
    let sign_in = json!({ "email": "ada@example.org", "password": PASSWORD });
    let routes = [("sign-up", sign_up), ("sign-in", sign_in)];
    for (route, body) in routes {
        let path = format!("/api/v1/auth/{route}");
        let builder = request(Method::POST, &path).header(ORIGIN, OTHER_ORIGIN);
        let answer = server.send(with_json(builder, &body)).await;
        answer.assert_problem(403, "auth.csrf");
    }
    assert!(
        server.accounts.mailer.sent().is_empty(),
        "no account was made"
    );
}

#[tokio::test]
async fn an_ended_sessions_cookie_needs_an_allowed_origin_but_no_token() {
    let server = TestServer::new();
    let stale = Browser {
        cookie: "A".repeat(43),
        csrf: String::new(),
    };
    let body = json!({ "email": "ada@example.org" });
    let path = "/api/v1/auth/password-reset";
    let from_app = with_json(stale.cookie(from_app(Method::POST, path)), &body);
    assert_eq!(server.send(from_app).await.status, 202);
    let elsewhere = request(Method::POST, path).header(ORIGIN, OTHER_ORIGIN);
    let answer = server.send(with_json(stale.cookie(elsewhere), &body)).await;
    answer.assert_problem(403, "auth.csrf");
}
