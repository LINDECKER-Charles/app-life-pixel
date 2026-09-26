//! Signing in with a password and a TOTP code, the session's expiry, CSRF and disabling.

use axum::http::StatusCode;
use time::Duration;

use crate::support::{PASSWORD, Stack};

#[tokio::test]
async fn signing_in_opens_a_session_the_cookie_carries() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let code = stack.code(&admin, 0);
    let (status, body, cookie) = stack.sign_in(("OPS@example.org", PASSWORD, &code)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["admin"]["email"], "ops@example.org");
    assert_eq!(
        body["admin"]["id"],
        admin.identity.id.hyphenated().to_string()
    );
    assert_eq!(body["environment"], "local");
    assert_eq!(
        body["monitoredEnvironments"],
        serde_json::json!(["staging", "production"])
    );
    let session = crate::support::Session {
        cookie: cookie.unwrap(),
        csrf_token: body["csrfToken"].as_str().unwrap().to_owned(),
    };
    assert!(session.cookie.starts_with("__Host-lpa_session="));
    let (status, read, _) = stack.session(&session).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read["csrfToken"], body["csrfToken"]);
    stack.drop().await;
}

#[tokio::test]
async fn any_wrong_part_is_the_same_refusal() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let code = stack.code(&admin, 0);
    let wrong_code = stack.code(&admin, 3);
    for (email, password, code) in [
        ("nobody@example.org", PASSWORD, code.as_str()),
        ("ops@example.org", "not the password", code.as_str()),
        ("ops@example.org", PASSWORD, wrong_code.as_str()),
        ("ops@example.org", PASSWORD, "12345"),
    ] {
        let (status, body, cookie) = stack.sign_in((email, password, code)).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{email} {password} {code}"
        );
        assert_eq!(body["code"], "admin.invalid_credentials");
        assert!(cookie.is_none());
    }
    stack.drop().await;
}

/// The status of `ops@example.org` signing in with `code`.
async fn signs_in_with(stack: &Stack, code: &str) -> StatusCode {
    stack.sign_in(("ops@example.org", PASSWORD, code)).await.0
}

#[tokio::test]
async fn a_code_signs_in_once_and_the_previous_step_is_allowed() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let previous = stack.code(&admin, -1);
    let status = signs_in_with(&stack, &previous).await;
    assert_eq!(status, StatusCode::OK, "one step of drift");
    let (status, body, _) = stack
        .sign_in(("ops@example.org", PASSWORD, &previous))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "a replayed code");
    assert_eq!(body["code"], "admin.invalid_credentials");
    let current = stack.code(&admin, 0);
    assert_eq!(signs_in_with(&stack, &current).await, StatusCode::OK);
    let status = signs_in_with(&stack, &previous).await;
    let message = "an older step after a newer one";
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{message}");
    stack.clock.advance(Duration::seconds(30));
    let next = stack.code(&admin, 0);
    assert_eq!(signs_in_with(&stack, &next).await, StatusCode::OK);
    stack.drop().await;
}

#[tokio::test]
async fn a_session_ends_after_thirty_idle_minutes() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let session = stack.signed_in(&admin).await;
    stack.clock.advance(Duration::minutes(29));
    assert_eq!(stack.session(&session).await.0, StatusCode::OK);
    stack.clock.advance(Duration::minutes(29));
    assert_eq!(
        stack.session(&session).await.0,
        StatusCode::OK,
        "each request keeps it alive"
    );
    stack.clock.advance(Duration::minutes(31));
    let (status, body, clears_cookie) = stack.session(&session).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "admin.unauthenticated");
    assert!(clears_cookie);
    stack.drop().await;
}

#[tokio::test]
async fn a_session_ends_after_eight_hours_however_active() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let session = stack.signed_in(&admin).await;
    for _ in 0..16 {
        stack.clock.advance(Duration::minutes(29));
        assert_eq!(stack.session(&session).await.0, StatusCode::OK);
    }
    stack.clock.advance(Duration::minutes(20));
    assert_eq!(stack.session(&session).await.0, StatusCode::UNAUTHORIZED);
    stack.drop().await;
}

#[tokio::test]
async fn a_change_needs_the_session_s_csrf_token() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let session = stack.signed_in(&admin).await;
    for token in [None, Some("forged"), Some(&session.csrf_token[1..])] {
        let (status, body) = stack
            .post("/api/admin/v1/users/0190/suspend", (&session, token))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{token:?}");
        assert_eq!(body["code"], "admin.csrf");
    }
    let token = Some(session.csrf_token.as_str());
    let (status, body) = stack
        .post("/api/admin/v1/users/0190/suspend", (&session, token))
        .await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "relayed to a server that is down"
    );
    assert_eq!(body["code"], "service.unavailable");
    let (status, _) = stack
        .post("/api/admin/v1/auth/sign-out", (&session, token))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(stack.session(&session).await.0, StatusCode::UNAUTHORIZED);
    stack.drop().await;
}

#[tokio::test]
async fn disabling_an_admin_ends_its_sessions_and_its_sign_ins() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let session = stack.signed_in(&admin).await;
    assert!(stack.state.admins.disable("OPS@example.org").await.unwrap());
    assert_eq!(stack.session(&session).await.0, StatusCode::UNAUTHORIZED);
    stack.clock.advance(Duration::seconds(30));
    let code = stack.code(&admin, 0);
    let (status, _, _) = stack.sign_in(("ops@example.org", PASSWORD, &code)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(!stack.state.admins.disable("ops@example.org").await.unwrap());
    stack.drop().await;
}
