//! `POST /auth/password-reset`, `POST /auth/password-reset/confirm` and `PUT /auth/password`.

use axum::body::Body;
use axum::http::{Method, Request};
use life_pixel_service::accounts::ports::Message;
use serde_json::json;
use time::Duration;

use crate::common::TestServer;
use crate::common::accounts::token_of;
use crate::common::auth::{
    APP_ORIGIN, Browser, NEW_PASSWORD, PASSWORD, from_app, sign_in, sign_up, with_json,
};

const SESSION: &str = "/api/v1/auth/session";

/// A server with one account and two of its sessions.
async fn two_sessions() -> (TestServer, Browser, Browser) {
    let server = TestServer::new();
    let first = Browser::of(&server.send(sign_up("ada@example.org", "en")).await);
    let second = Browser::of(&server.send(sign_in("ada@example.org", PASSWORD)).await);
    (server, first, second)
}

/// A request for a reset link sent to `email`.
fn reset(email: &str) -> Request<Body> {
    let path = "/api/v1/auth/password-reset";
    with_json(from_app(Method::POST, path), &json!({ "email": email }))
}

/// The new password `password`, with the token of a reset link.
fn confirm(token: &str, password: &str) -> Request<Body> {
    let path = "/api/v1/auth/password-reset/confirm";
    let body = json!({ "token": token, "password": password });
    with_json(from_app(Method::POST, path), &body)
}

#[tokio::test]
async fn a_reset_link_sets_a_new_password_once_and_ends_every_session() {
    let (server, first, second) = two_sessions().await;
    assert_eq!(server.send(reset(" ADA@example.org")).await.status, 202);
    let email = server.accounts.email(2).await;
    let link = email.message.link().unwrap().to_owned();
    let expected = format!("{APP_ORIGIN}/reset-password/confirm?token=");
    assert!(link.starts_with(&expected), "{link}");
    let token = token_of(&link);
    assert_eq!(server.send(confirm(&token, NEW_PASSWORD)).await.status, 204);
    for browser in [first, second] {
        let session = server.send(browser.get(SESSION)).await;
        session.assert_problem(401, "auth.unauthenticated");
    }
    assert_eq!(
        server.accounts.email(3).await.message,
        Message::PasswordChanged
    );
    let old = server.send(sign_in("ada@example.org", PASSWORD)).await;
    old.assert_problem(401, "auth.invalid_credentials");
    let new = server.send(sign_in("ada@example.org", NEW_PASSWORD)).await;
    assert_eq!(new.status, 200);
    let again = server.send(confirm(&token, PASSWORD)).await;
    again.assert_problem(400, "auth.token_invalid");
}

#[tokio::test]
async fn a_reset_is_accepted_alike_for_an_address_without_an_account() {
    let (server, _, _) = two_sessions().await;
    for email in ["nobody@example.org", "not an address"] {
        assert_eq!(server.send(reset(email)).await.status, 202);
    }
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        server.accounts.mailer.sent().len(),
        1,
        "the verification email alone"
    );
}

#[tokio::test]
async fn a_reset_link_expires_after_an_hour() {
    let (server, _, _) = two_sessions().await;
    assert_eq!(server.send(reset("ada@example.org")).await.status, 202);
    let token = server.accounts.token(2).await;
    let clock = &server.accounts.clock;
    clock.advance(Duration::hours(1) + Duration::seconds(1));
    let late = server.send(confirm(&token, NEW_PASSWORD)).await;
    late.assert_problem(400, "auth.token_invalid");
    let unknown = server.send(confirm(&"A".repeat(43), NEW_PASSWORD)).await;
    unknown.assert_problem(400, "auth.token_invalid");
}

#[tokio::test]
async fn a_new_password_of_the_wrong_length_leaves_the_link_working() {
    let (server, _, _) = two_sessions().await;
    assert_eq!(server.send(reset("ada@example.org")).await.status, 202);
    let token = server.accounts.token(2).await;
    let short = server.send(confirm(&token, "too short")).await;
    let params = short.assert_problem(422, "auth.password_length");
    assert_eq!(params, json!({ "min": 12, "max": 128 }));
    assert_eq!(server.send(confirm(&token, NEW_PASSWORD)).await.status, 204);
}

#[tokio::test]
async fn changing_the_password_ends_the_other_sessions() {
    let (server, first, second) = two_sessions().await;
    let answer = server
        .send(first.change_password(PASSWORD, NEW_PASSWORD))
        .await;
    assert_eq!(answer.status, 204, "{:?}", answer.body);
    assert_eq!(server.send(first.get(SESSION)).await.status, 200);
    let other = server.send(second.get(SESSION)).await;
    other.assert_problem(401, "auth.unauthenticated");
    assert_eq!(
        server.accounts.email(2).await.message,
        Message::PasswordChanged
    );
    let new = server.send(sign_in("ada@example.org", NEW_PASSWORD)).await;
    assert_eq!(new.status, 200);
}

#[tokio::test]
async fn a_change_needs_the_current_password_a_valid_new_one_and_a_session() {
    let (server, first, second) = two_sessions().await;
    let wrong = server
        .send(first.change_password("wrong password!", NEW_PASSWORD))
        .await;
    assert_eq!(
        wrong.assert_problem(403, "auth.current_password"),
        json!({})
    );
    let short = server
        .send(first.change_password(PASSWORD, "too short"))
        .await;
    short.assert_problem(422, "auth.password_length");
    assert_eq!(server.send(second.get(SESSION)).await.status, 200);
    let body = json!({ "currentPassword": PASSWORD, "newPassword": NEW_PASSWORD });
    let anonymous = with_json(from_app(Method::PUT, "/api/v1/auth/password"), &body);
    let answer = server.send(anonymous).await;
    answer.assert_problem(401, "auth.unauthenticated");
}
