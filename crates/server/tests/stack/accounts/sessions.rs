//! Sessions over Postgres: their cookie, sliding expiry, CSRF, the ends a reset and a change
//! bring, suspension, and the daily purge.

use axum::http::Method;
use axum::http::header::ORIGIN;
use life_pixel_server::testing::TestMailbox;
use serde_json::json;

use super::{AccountsStack, post};
use crate::router::auth::{
    Browser, CLEARED_COOKIE, COOKIE_ATTRIBUTES, NEW_PASSWORD, OTHER_ORIGIN, PASSWORD, from_app,
    sign_in, with_json,
};
use crate::router::request;

const SESSION: &str = "/api/v1/auth/session";
const AGE_SESSIONS: &str = "update sessions set last_seen_at = last_seen_at - interval '25 hours', \
                            expires_at = expires_at - interval '25 hours' where account_id = \
                            (select id from accounts where email = $1::citext)";
const EXPIRES_A_MONTH_AWAY: &str = "select bool_and(expires_at > now() + interval '29 days \
                                    23 hours') from sessions where account_id = \
                                    (select id from accounts where email = $1::citext)";
const EXPIRE_SESSIONS: &str = "update sessions set expires_at = now() - interval '1 second' \
                               where account_id = (select id from accounts where email = \
                               $1::citext)";
const SUSPEND: &str = "update accounts set status = 'suspended' where email = $1::citext";
const HASHED_SESSIONS: &str = "select count(*) from sessions where octet_length(token_hash) = 32 \
                               and account_id = (select id from accounts where email = \
                               $1::citext)";

/// A test database with one account, and its session.
async fn signed_up() -> (AccountsStack, TestMailbox, Browser) {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let browser = stack.sign_up(&mailbox, "en").await;
    (stack, mailbox, browser)
}

#[tokio::test]
async fn a_session_seen_again_after_a_day_lives_thirty_days_more() {
    let (stack, mailbox, browser) = signed_up().await;
    let soon = stack.send(browser.get(SESSION)).await;
    assert!(soon.headers.get("set-cookie").is_none());
    assert_eq!(stack.execute(AGE_SESSIONS, mailbox.address()).await, 1);
    let aged: bool = stack.select(EXPIRES_A_MONTH_AWAY, mailbox.address()).await;
    assert!(!aged);
    let later = stack.send(browser.get(SESSION)).await;
    let renewed = format!("__Host-lp_session={}; {COOKIE_ATTRIBUTES}", browser.cookie);
    assert_eq!(later.header("set-cookie"), renewed);
    let moved: bool = stack.select(EXPIRES_A_MONTH_AWAY, mailbox.address()).await;
    assert!(moved);
    let again = stack.send(browser.get(SESSION)).await;
    assert!(again.headers.get("set-cookie").is_none());
    let hashed: i64 = stack.select(HASHED_SESSIONS, mailbox.address()).await;
    assert_eq!(hashed, 1, "the session is stored as its SHA-256");
}

#[tokio::test]
async fn an_expired_session_ends_and_the_purge_removes_it() {
    let (stack, mailbox, browser) = signed_up().await;
    assert_eq!(stack.execute(EXPIRE_SESSIONS, mailbox.address()).await, 1);
    let answer = stack.send(browser.get(SESSION)).await;
    answer.assert_problem(401, "auth.unauthenticated");
    assert_eq!(answer.header("set-cookie"), CLEARED_COOKIE);
    let (sessions, _) = stack.state.accounts.purge_expired().await.unwrap();
    assert_eq!(sessions, 1);
    let left: i64 = stack.select(HASHED_SESSIONS, mailbox.address()).await;
    assert_eq!(left, 0);
}

#[tokio::test]
async fn an_unsafe_request_needs_its_token_and_an_allowed_origin() {
    let (stack, mailbox, first) = signed_up().await;
    let second = stack.sign_in(&mailbox).await;
    let change = json!({ "currentPassword": PASSWORD, "newPassword": NEW_PASSWORD });
    let path = "/api/v1/auth/password";
    let without = with_json(first.cookie(from_app(Method::PUT, path)), &change);
    stack.send(without).await.assert_problem(403, "auth.csrf");
    let borrowed = Browser {
        cookie: first.cookie.clone(),
        csrf: second.csrf.clone(),
    };
    let wrong = stack
        .send(borrowed.change_password(PASSWORD, NEW_PASSWORD))
        .await;
    wrong.assert_problem(403, "auth.csrf");
    let elsewhere = request(Method::PUT, path).header(ORIGIN, OTHER_ORIGIN);
    let answer = stack
        .send(with_json(first.signed(elsewhere), &change))
        .await;
    answer.assert_problem(403, "auth.csrf");
    let unchanged = stack.send(sign_in(mailbox.address(), PASSWORD)).await;
    assert_eq!(unchanged.status, 200);
}

#[tokio::test]
async fn a_change_ends_the_other_sessions_and_a_reset_every_one() {
    let (stack, mailbox, first) = signed_up().await;
    let second = stack.sign_in(&mailbox).await;
    let changed = first.change_password(PASSWORD, NEW_PASSWORD);
    assert_eq!(stack.status(changed).await, 204);
    assert_eq!(stack.status(first.get(SESSION)).await, 200);
    let other = stack.send(second.get(SESSION)).await;
    other.assert_problem(401, "auth.unauthenticated");
    let third = sign_in(mailbox.address(), NEW_PASSWORD);
    let third = Browser::of(&stack.send(third).await);
    let token = stack.reset_token(&mailbox, "en").await;
    let confirm = json!({ "token": token, "password": PASSWORD });
    assert_eq!(
        stack.status(post("password-reset/confirm", &confirm)).await,
        204
    );
    for browser in [first, third] {
        let session = stack.send(browser.get(SESSION)).await;
        session.assert_problem(401, "auth.unauthenticated");
    }
}

#[tokio::test]
async fn a_suspended_account_cannot_sign_in_and_its_sessions_stop_working() {
    let (stack, mailbox, browser) = signed_up().await;
    assert_eq!(stack.execute(SUSPEND, mailbox.address()).await, 1);
    let session = stack.send(browser.get(SESSION)).await;
    session.assert_problem(403, "auth.account_suspended");
    let resend = browser.post("/api/v1/auth/verify-email/resend", &json!({}));
    stack
        .send(resend)
        .await
        .assert_problem(403, "auth.account_suspended");
    let right = stack.send(sign_in(mailbox.address(), PASSWORD)).await;
    right.assert_problem(403, "auth.account_suspended");
    let wrong = stack
        .send(sign_in(mailbox.address(), "wrong password!"))
        .await;
    wrong.assert_problem(401, "auth.invalid_credentials");
}
