//! `POST /auth/sign-up`: a new account, signed in at once, and its verification email.

use axum::http::Method;
use serde_json::{Value, json};

use crate::common::auth::{APP_ORIGIN, Browser, PASSWORD, from_app, peer, sign_up, with_json};
use crate::common::{TestServer, send};

const SIGN_UP: &str = "/api/v1/auth/sign-up";
const ADA: &str = "ada@example.org";
const EMAIL_INVALID: &str = "auth.email_invalid";
const PASSWORD_LENGTH: &str = "auth.password_length";

/// A sign-up of `email`, `password` and `language`.
fn body(email: &str, password: &str, language: &str) -> Value {
    json!({ "email": email, "password": password, "language": language })
}

#[tokio::test]
async fn signing_up_opens_a_session_and_emails_a_verification_link() {
    let server = TestServer::new();
    let answer = server.send(sign_up("  Ada@Example.org ", "fr")).await;
    assert_eq!(answer.status, 201, "{:?}", answer.body);
    let browser = Browser::of(&answer);
    let account = answer.json()["account"].clone();
    assert_eq!(account["email"], "Ada@Example.org");
    assert_eq!(account["emailVerified"], false);
    assert_eq!(account["language"], "fr");
    assert_eq!(account["plan"], "free");
    let storage = json!({ "usedBytes": 0, "limitBytes": 100_000_000 });
    assert_eq!(account["storage"], storage);
    assert!(account["createdAt"].as_str().unwrap().ends_with('Z'));
    let email = server.accounts.email(1).await;
    assert_eq!(
        (email.to.as_str(), email.language.as_str()),
        ("Ada@Example.org", "fr")
    );
    let link = email.message.link().unwrap();
    assert!(
        link.starts_with(&format!("{APP_ORIGIN}/verify-email?token=")),
        "{link}"
    );
    let session = server.send(browser.get("/api/v1/auth/session")).await;
    assert_eq!(session.json()["account"], account);
}

#[tokio::test]
async fn an_invalid_sign_up_says_why() {
    let server = TestServer::new();
    let long = "x".repeat(129);
    let cases = [
        ("ada.example.org", PASSWORD, "en", EMAIL_INVALID),
        ("ada@example", PASSWORD, "en", EMAIL_INVALID),
        ("@example.org", PASSWORD, "en", EMAIL_INVALID),
        ("ada@@example.org", PASSWORD, "en", EMAIL_INVALID),
        (ADA, "eleven char", "en", PASSWORD_LENGTH),
        (ADA, &long, "en", PASSWORD_LENGTH),
        (ADA, PASSWORD, "de", "account.language"),
    ];
    for (n, (email, password, language, code)) in (1..).zip(cases) {
        let body = body(email, password, language);
        let request = with_json(from_app(Method::POST, SIGN_UP), &body);
        let answer = send(server.public_router(peer(n)), request).await;
        let params = answer.assert_problem(422, code);
        let expected = match code {
            PASSWORD_LENGTH => json!({ "min": 12, "max": 128 }),
            "account.language" => json!({ "available": ["en", "fr"] }),
            _ => json!({}),
        };
        assert_eq!(params, expected, "{code}");
    }
    assert!(server.accounts.mailer.sent().is_empty());
}

#[tokio::test]
async fn an_address_is_taken_whatever_its_case() {
    let server = TestServer::new();
    assert_eq!(
        server.send(sign_up("ada@example.org", "en")).await.status,
        201
    );
    let again = server.send(sign_up(" ADA@example.ORG", "fr")).await;
    again.assert_problem(409, "auth.email_taken");
}

#[tokio::test]
async fn signing_up_is_limited_to_five_an_hour_per_address() {
    let server = TestServer::new();
    for _ in 0..5 {
        let answer = server.send(sign_up("not an address", "en")).await;
        answer.assert_problem(422, "auth.email_invalid");
    }
    let answer = server.send(sign_up("ada@example.org", "en")).await;
    answer.assert_problem(429, "rate_limit.exceeded");
    let elsewhere = send(
        server.public_router(peer(1)),
        sign_up("ada@example.org", "en"),
    );
    assert_eq!(elsewhere.await.status, 201);
}
