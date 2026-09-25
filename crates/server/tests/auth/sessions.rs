//! `GET /auth/session` and `POST /auth/sign-out`: the cookie, its sliding expiry, and its end.

use axum::http::Method;
use serde_json::json;
use time::Duration;

use crate::common::auth::{
    Browser, CLEARED_COOKIE, COOKIE_ATTRIBUTES, PASSWORD, sign_in, sign_up, with_json, without_body,
};
use crate::common::{Answer, TestServer, request};

const SESSION: &str = "/api/v1/auth/session";
const SIGN_OUT: &str = "/api/v1/auth/sign-out";

/// A server with one account, and its session.
async fn signed_up() -> (TestServer, Browser) {
    let server = TestServer::new();
    let answer = server.send(sign_up("ada@example.org", "en")).await;
    assert_eq!(answer.status, 201);
    (server, Browser::of(&answer))
}

/// The `Set-Cookie` of `answer`, if any.
fn set_cookie(answer: &Answer) -> Option<&str> {
    let value = answer.headers.get("set-cookie")?;
    Some(value.to_str().unwrap())
}

#[tokio::test]
async fn the_session_cookie_is_bound_to_the_host_and_hidden_from_scripts() {
    let server = TestServer::new();
    let answer = server.send(sign_up("ada@example.org", "en")).await;
    let set = answer.header("set-cookie");
    assert!(set.starts_with("__Host-lp_session="), "{set}");
    assert!(set.ends_with(&format!("; {COOKIE_ATTRIBUTES}")), "{set}");
    let csrf = answer.json()["csrfToken"].as_str().unwrap().to_owned();
    assert_eq!(csrf.len(), 43, "an HMAC-SHA256 in base64url: {csrf}");
}

#[tokio::test]
async fn without_a_live_session_the_session_is_unauthenticated() {
    let server = TestServer::new();
    let none = server
        .send(without_body(request(Method::GET, SESSION)))
        .await;
    none.assert_problem(401, "auth.unauthenticated");
    assert_eq!(set_cookie(&none), None);
    for cookie in ["A".repeat(43), "not-a-token".to_owned()] {
        let stale = Browser {
            cookie,
            csrf: String::new(),
        };
        let answer = server.send(stale.get(SESSION)).await;
        answer.assert_problem(401, "auth.unauthenticated");
        assert_eq!(set_cookie(&answer), Some(CLEARED_COOKIE));
    }
}

#[tokio::test]
async fn a_session_seen_again_after_a_day_lives_thirty_days_more() {
    let (server, browser) = signed_up().await;
    let clock = &server.accounts.clock;
    clock.advance(Duration::hours(12));
    let soon = server.send(browser.get(SESSION)).await;
    assert_eq!((soon.status.as_u16(), set_cookie(&soon)), (200, None));
    clock.advance(Duration::hours(13));
    let later = server.send(browser.get(SESSION)).await;
    let renewed = format!("__Host-lp_session={}; {COOKIE_ATTRIBUTES}", browser.cookie);
    assert_eq!(set_cookie(&later), Some(renewed.as_str()));
    assert_eq!(set_cookie(&server.send(browser.get(SESSION)).await), None);
    clock.advance(Duration::days(29) + Duration::hours(23));
    let month_later = server.send(browser.get(SESSION)).await;
    assert_eq!(
        month_later.status, 200,
        "past the first expiry, before the moved one"
    );
    assert_eq!(set_cookie(&month_later), Some(renewed.as_str()));
}

#[tokio::test]
async fn a_session_unseen_for_thirty_days_ends() {
    let (server, browser) = signed_up().await;
    server
        .accounts
        .clock
        .advance(Duration::days(30) + Duration::seconds(1));
    let answer = server.send(browser.get(SESSION)).await;
    answer.assert_problem(401, "auth.unauthenticated");
    assert_eq!(set_cookie(&answer), Some(CLEARED_COOKIE));
}

#[tokio::test]
async fn signing_out_ends_the_session_and_clears_its_cookie() {
    let (server, browser) = signed_up().await;
    let answer = server.send(browser.post(SIGN_OUT, &json!({}))).await;
    assert_eq!(answer.status, 204, "{:?}", answer.body);
    assert_eq!(set_cookie(&answer), Some(CLEARED_COOKIE));
    let after = server.send(browser.get(SESSION)).await;
    after.assert_problem(401, "auth.unauthenticated");
    let again = server.send(sign_in("ada@example.org", PASSWORD)).await;
    assert_eq!(again.status, 200, "the account stays");
}

#[tokio::test]
async fn signing_out_without_a_session_still_clears_the_cookie() {
    let server = TestServer::new();
    let bare = request(Method::POST, SIGN_OUT);
    let answer = server.send(with_json(bare, &json!({}))).await;
    assert_eq!(answer.status, 204);
    assert_eq!(set_cookie(&answer), Some(CLEARED_COOKIE));
}
