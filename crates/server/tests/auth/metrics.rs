//! `auth_events_total{event}`: one count per sign-up, sign-in, failed sign-in, sign-out, reset
//! and change of password. The only test of this binary to install the recorder: the other
//! tests count too, once it is.

use axum::http::Method;
use life_pixel_server::accounts::metrics;
use life_pixel_server::{app, telemetry};
use serde_json::json;

use crate::common::auth::{Browser, NEW_PASSWORD, PASSWORD, sign_in, sign_up};
use crate::common::{TestServer, empty, send};

/// The events, as their label.
const EVENTS: [&str; 6] = [
    "sign_up",
    "sign_in",
    "sign_in_failed",
    "sign_out",
    "password_reset",
    "password_changed",
];

/// Signs up, fails to sign in, changes the password, signs out, signs in and resets the
/// password.
async fn every_event(server: &TestServer) {
    let browser = Browser::of(&server.send(sign_up("ada@example.org", "en")).await);
    let mut statuses = Vec::new();
    let failed = sign_in("ada@example.org", "wrong password!");
    statuses.push(server.send(failed).await.status);
    let change = browser.change_password(PASSWORD, NEW_PASSWORD);
    statuses.push(server.send(change).await.status);
    let sign_out = browser.post("/api/v1/auth/sign-out", &json!({}));
    statuses.push(server.send(sign_out).await.status);
    let again = sign_in("ada@example.org", NEW_PASSWORD);
    let again = Browser::of(&server.send(again).await);
    let reset = json!({ "email": "ada@example.org" });
    let reset = again.post("/api/v1/auth/password-reset", &reset);
    statuses.push(server.send(reset).await.status);
    let confirm = json!({ "token": server.accounts.token(3).await, "password": PASSWORD });
    let confirm = again.post("/api/v1/auth/password-reset/confirm", &confirm);
    statuses.push(server.send(confirm).await.status);
    assert_eq!(statuses, [401, 204, 204, 202, 204]);
}

#[tokio::test]
async fn each_auth_event_is_counted() {
    let handle = telemetry::recorder().unwrap_or_else(|error| panic!("recorder: {error}"));
    metrics::describe();
    every_event(&TestServer::new()).await;
    let answer = send(app::metrics_router(handle), empty(Method::GET, "/metrics")).await;
    let text = String::from_utf8_lossy(&answer.body).into_owned();
    for event in EVENTS {
        let series = format!("auth_events_total{{event=\"{event}\"}} ");
        assert!(
            text.lines().any(|line| line.starts_with(&series)),
            "{series} in {text}"
        );
    }
}
