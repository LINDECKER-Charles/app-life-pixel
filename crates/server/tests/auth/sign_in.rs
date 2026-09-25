//! `POST /auth/sign-in`: a session for an address and its password, never saying which was
//! wrong.

use axum::http::Method;
use axum::http::header::ORIGIN;
use serde_json::json;

use crate::common::auth::{Browser, DEV_ORIGIN, PASSWORD, peer, sign_in, sign_up, with_json};
use crate::common::{TestServer, request, send};

const SESSION: &str = "/api/v1/auth/session";

/// A server with the account `ada@example.org`, and its sign-up's session.
async fn server_with_ada() -> (TestServer, Browser) {
    let server = TestServer::new();
    let answer = server.send(sign_up("ada@example.org", "en")).await;
    assert_eq!(answer.status, 201);
    (server, Browser::of(&answer))
}

#[tokio::test]
async fn signing_in_opens_another_session() {
    let (server, first) = server_with_ada().await;
    let answer = server.send(sign_in("ADA@example.org", PASSWORD)).await;
    assert_eq!(answer.status, 200, "{:?}", answer.body);
    let second = Browser::of(&answer);
    assert_ne!(second.cookie, first.cookie);
    assert_ne!(second.csrf, first.csrf);
    assert_eq!(answer.json()["account"]["email"], "ada@example.org");
    for browser in [first, second] {
        let session = server.send(browser.get(SESSION)).await;
        assert_eq!(session.json()["csrfToken"], browser.csrf.as_str());
    }
}

#[tokio::test]
async fn a_wrong_password_and_an_unknown_address_answer_alike() {
    let (server, _) = server_with_ada().await;
    let attempts = [
        ("ada@example.org", "wrong password!"),
        ("nobody@example.org", PASSWORD),
        ("not an address", PASSWORD),
        ("ada@example.org", "short"),
    ];
    for (n, (email, password)) in (1..).zip(attempts) {
        let answer = send(server.public_router(peer(n)), sign_in(email, password)).await;
        assert_eq!(
            answer.assert_problem(401, "auth.invalid_credentials"),
            json!({})
        );
        assert!(answer.headers.get("set-cookie").is_none());
    }
}

#[tokio::test]
async fn a_suspended_account_cannot_sign_in_and_its_sessions_stop_working() {
    let (server, browser) = server_with_ada().await;
    let id = server.send(browser.get(SESSION)).await.json()["account"]["id"].clone();
    server.accounts.suspend(&id);
    let session = server.send(browser.get(SESSION)).await;
    session.assert_problem(403, "auth.account_suspended");
    assert!(session.headers.get("set-cookie").is_none());
    let right = server.send(sign_in("ada@example.org", PASSWORD)).await;
    right.assert_problem(403, "auth.account_suspended");
    let wrong = server
        .send(sign_in("ada@example.org", "wrong password!"))
        .await;
    wrong.assert_problem(401, "auth.invalid_credentials");
}

#[tokio::test]
async fn signing_in_is_limited_per_address_signed_in_to() {
    let (server, _) = server_with_ada().await;
    for n in 1..=5 {
        let answer = send(
            server.public_router(peer(n)),
            sign_in("ada@example.org", "wrong!"),
        );
        answer.await.assert_problem(401, "auth.invalid_credentials");
    }
    let sixth = send(
        server.public_router(peer(6)),
        sign_in("Ada@example.org", PASSWORD),
    );
    sixth.await.assert_problem(429, "rate_limit.exceeded");
}

#[tokio::test]
async fn signing_in_from_an_allowed_origin_works_and_without_one_is_refused() {
    let (server, _) = server_with_ada().await;
    let body = json!({ "email": "ada@example.org", "password": PASSWORD });
    let from_dev = request(Method::POST, "/api/v1/auth/sign-in").header(ORIGIN, DEV_ORIGIN);
    assert_eq!(server.send(with_json(from_dev, &body)).await.status, 200);
    let anonymous = request(Method::POST, "/api/v1/auth/sign-in");
    let answer = server.send(with_json(anonymous, &body)).await;
    answer.assert_problem(403, "auth.csrf");
}
