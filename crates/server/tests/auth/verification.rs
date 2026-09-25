//! `POST /auth/verify-email` and `POST /auth/verify-email/resend`.

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::json;
use time::Duration;

use crate::common::TestServer;
use crate::common::auth::{Browser, from_app, sign_up, with_json};

const SESSION: &str = "/api/v1/auth/session";
const RESEND: &str = "/api/v1/auth/verify-email/resend";

/// A server with one account, its session, and the token of its verification link.
async fn signed_up() -> (TestServer, Browser, String) {
    let server = TestServer::new();
    let browser = Browser::of(&server.send(sign_up("ada@example.org", "en")).await);
    let token = server.accounts.token(1).await;
    (server, browser, token)
}

/// The verification of the link of `token`.
fn verify(token: &str) -> Request<Body> {
    let body = json!({ "token": token });
    with_json(from_app(Method::POST, "/api/v1/auth/verify-email"), &body)
}

#[tokio::test]
async fn a_verification_link_verifies_the_address_once() {
    let (server, browser, token) = signed_up().await;
    assert_eq!(server.send(verify(&token)).await.status, 204);
    let session = server.send(browser.get(SESSION)).await;
    assert_eq!(session.json()["account"]["emailVerified"], true);
    let again = server.send(verify(&token)).await;
    assert_eq!(again.assert_problem(400, "auth.token_invalid"), json!({}));
}

#[tokio::test]
async fn a_verification_link_expires_after_seven_days() {
    let (server, _, token) = signed_up().await;
    let clock = &server.accounts.clock;
    clock.advance(Duration::days(7) + Duration::seconds(1));
    let late = server.send(verify(&token)).await;
    late.assert_problem(400, "auth.token_invalid");
    let garbled = server.send(verify("not a token")).await;
    garbled.assert_problem(400, "auth.token_invalid");
}

#[tokio::test]
async fn a_new_verification_link_can_be_asked_for() {
    let (server, browser, first) = signed_up().await;
    let answer = server.send(browser.post(RESEND, &json!({}))).await;
    assert_eq!(answer.status, 202, "{:?}", answer.body);
    let second = server.accounts.token(2).await;
    assert_ne!(second, first);
    assert_eq!(server.send(verify(&second)).await.status, 204);
    assert_eq!(
        server.send(browser.post(RESEND, &json!({}))).await.status,
        202
    );
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let sent = server.accounts.mailer.sent().len();
    assert_eq!(sent, 2, "none once the address is verified");
}

#[tokio::test]
async fn asking_for_a_new_link_needs_a_session_and_is_limited_to_three_an_hour() {
    let (server, browser, _) = signed_up().await;
    let anonymous = with_json(from_app(Method::POST, RESEND), &json!({}));
    let answer = server.send(anonymous).await;
    answer.assert_problem(401, "auth.unauthenticated");
    for _ in 0..3 {
        let answer = server.send(browser.post(RESEND, &json!({}))).await;
        assert_eq!(answer.status, 202);
    }
    let fourth = server.send(browser.post(RESEND, &json!({}))).await;
    fourth.assert_problem(429, "rate_limit.exceeded");
}
