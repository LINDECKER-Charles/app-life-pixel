//! Each auth route over Postgres and Mailpit: its success, and each of its problems.

use axum::http::Method;
use axum::http::header::ORIGIN;
use life_pixel_server::testing::TestMailbox;
use serde_json::{Value, json};

use super::{AccountsStack, message, messages, post, text, token_in};
use crate::router::auth::{
    Browser, CLEARED_COOKIE, NEW_PASSWORD, OTHER_ORIGIN, PASSWORD, from_app, peer, sign_in,
    sign_up, with_json, without_body,
};
use crate::router::request;

const SESSION: &str = "/api/v1/auth/session";

/// A sign-up of `email`, `password` and `language`.
fn sign_up_body(email: &str, password: &str, language: &str) -> Value {
    json!({ "email": email, "password": password, "language": language })
}

#[tokio::test]
async fn sign_up_creates_the_account_once_per_address() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let answer = stack.send(sign_up(mailbox.address(), "en")).await;
    assert_eq!(answer.status, 201, "{:?}", answer.body);
    let account = answer.json()["account"].clone();
    assert_eq!(account["email"], mailbox.address());
    assert_eq!(account["storage"]["limitBytes"], 100_000_000);
    let session = stack.send(Browser::of(&answer).get(SESSION)).await;
    assert_eq!(session.json()["account"], account);
    let taken = sign_up(&mailbox.address().to_uppercase(), "en");
    let answer = stack.send_from(peer(1), taken).await;
    answer.assert_problem(409, "auth.email_taken");
}

#[tokio::test]
async fn sign_up_says_what_is_wrong() {
    let stack = AccountsStack::new().await;
    let cases = [
        ("no-at.example", PASSWORD, "en", "auth.email_invalid"),
        ("ada@example.org", "short", "en", "auth.password_length"),
        ("ada@example.org", PASSWORD, "xx", "account.language"),
    ];
    for (n, (email, password, language, code)) in (1..).zip(cases) {
        let body = sign_up_body(email, password, language);
        let answer = stack.send_from(peer(n), post("sign-up", &body)).await;
        answer.assert_problem(422, code);
    }
    let elsewhere = request(Method::POST, "/api/v1/auth/sign-up").header(ORIGIN, OTHER_ORIGIN);
    let body = sign_up_body("ada@example.org", PASSWORD, "en");
    let answer = stack.send_from(peer(9), with_json(elsewhere, &body)).await;
    answer.assert_problem(403, "auth.csrf");
}

#[tokio::test]
async fn sign_in_opens_a_session_or_refuses_alike() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    stack.sign_up(&mailbox, "en").await;
    let upper = mailbox.address().to_uppercase();
    let answer = stack.send(sign_in(&upper, PASSWORD)).await;
    assert_eq!(answer.status, 200, "{:?}", answer.body);
    assert_eq!(stack.status(Browser::of(&answer).get(SESSION)).await, 200);
    let wrong = stack
        .send(sign_in(mailbox.address(), "wrong password!"))
        .await;
    wrong.assert_problem(401, "auth.invalid_credentials");
    let unknown = sign_in("nobody@test.life-pixel.invalid", PASSWORD);
    let unknown = stack.send(unknown).await;
    unknown.assert_problem(401, "auth.invalid_credentials");
    let elsewhere = request(Method::POST, "/api/v1/auth/sign-in").header(ORIGIN, OTHER_ORIGIN);
    let body = json!({ "email": mailbox.address(), "password": PASSWORD });
    let answer = stack.send(with_json(elsewhere, &body)).await;
    answer.assert_problem(403, "auth.csrf");
}

#[tokio::test]
async fn the_session_is_read_then_ended_by_signing_out() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let browser = stack.sign_up(&mailbox, "en").await;
    let session = stack.send(browser.get(SESSION)).await;
    assert_eq!(session.json()["csrfToken"], browser.csrf.as_str());
    let sign_out = browser.post("/api/v1/auth/sign-out", &json!({}));
    let answer = stack.send(sign_out).await;
    assert_eq!(answer.status, 204);
    assert_eq!(answer.header("set-cookie"), CLEARED_COOKIE);
    let after = stack.send(browser.get(SESSION)).await;
    after.assert_problem(401, "auth.unauthenticated");
    assert_eq!(after.header("set-cookie"), CLEARED_COOKIE);
    let none = stack
        .send(without_body(request(Method::GET, SESSION)))
        .await;
    none.assert_problem(401, "auth.unauthenticated");
}

#[tokio::test]
async fn an_address_is_verified_with_its_link_sent_again_on_demand() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let browser = stack.sign_up(&mailbox, "en").await;
    let subject = text("en", "email.verify_email.subject");
    message(&mailbox, &subject).await;
    let resend = browser.post("/api/v1/auth/verify-email/resend", &json!({}));
    assert_eq!(stack.status(resend).await, 202);
    let both = messages(&mailbox, &subject, 2).await;
    let token = token_in(&both[0], "/verify-email");
    assert_ne!(token, token_in(&both[1], "/verify-email"));
    let verify = post("verify-email", &json!({ "token": token }));
    assert_eq!(stack.status(verify).await, 204);
    let session = stack.send(browser.get(SESSION)).await;
    assert_eq!(session.json()["account"]["emailVerified"], true);
    let unknown = post("verify-email", &json!({ "token": "A".repeat(43) }));
    stack
        .send(unknown)
        .await
        .assert_problem(400, "auth.token_invalid");
    let anonymous = stack.send(post("verify-email/resend", &json!({}))).await;
    anonymous.assert_problem(401, "auth.unauthenticated");
}

#[tokio::test]
async fn a_password_is_reset_through_its_emailed_link() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    stack.sign_up(&mailbox, "en").await;
    let absent = json!({ "email": "nobody@test.life-pixel.invalid" });
    assert_eq!(stack.status(post("password-reset", &absent)).await, 202);
    let token = stack.reset_token(&mailbox, "en").await;
    let short = json!({ "token": token, "password": "short" });
    let short = stack.send(post("password-reset/confirm", &short)).await;
    short.assert_problem(422, "auth.password_length");
    let body = json!({ "token": token, "password": NEW_PASSWORD });
    assert_eq!(
        stack.status(post("password-reset/confirm", &body)).await,
        204
    );
    let again = stack.send(post("password-reset/confirm", &body)).await;
    again.assert_problem(400, "auth.token_invalid");
    let signed_in = sign_in(mailbox.address(), NEW_PASSWORD);
    assert_eq!(stack.status(signed_in).await, 200);
}

#[tokio::test]
async fn a_password_is_changed_with_the_current_one() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let browser = stack.sign_up(&mailbox, "en").await;
    let wrong = browser.change_password("wrong password!", NEW_PASSWORD);
    stack
        .send(wrong)
        .await
        .assert_problem(403, "auth.current_password");
    let short = browser.change_password(PASSWORD, "short");
    stack
        .send(short)
        .await
        .assert_problem(422, "auth.password_length");
    let changed = browser.change_password(PASSWORD, NEW_PASSWORD);
    assert_eq!(stack.status(changed).await, 204);
    let body = json!({ "currentPassword": NEW_PASSWORD, "newPassword": PASSWORD });
    let anonymous = with_json(from_app(Method::PUT, "/api/v1/auth/password"), &body);
    stack
        .send(anonymous)
        .await
        .assert_problem(401, "auth.unauthenticated");
    let signed_in = sign_in(mailbox.address(), NEW_PASSWORD);
    assert_eq!(stack.status(signed_in).await, 200);
}
