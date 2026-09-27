//! The internal admin API over the local stack (H10): the secret and the identity headers, each
//! route, each change audited in its own transaction — a failure leaving neither the change nor
//! its entry —, the log refusing updates and deletes, the reply emailed through Mailpit, and the
//! metrics on fixed data. Behind the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "admin_api/access.rs"]
mod access;
#[path = "admin_api/account_actions.rs"]
mod account_actions;
#[path = "admin_api/answers.rs"]
mod answers;
#[path = "admin_api/audit_log.rs"]
mod audit_log;
#[path = "admin_api/metrics.rs"]
mod metrics;
#[path = "admin_api/queue.rs"]
mod queue;
#[path = "common/mod.rs"]
mod router;
#[path = "common/stack.rs"]
mod stack;
#[path = "common/tokens.rs"]
mod tokens;
#[path = "admin_api/users.rs"]
mod users;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::request::Builder;
use axum::http::{Method, Request};
use life_pixel_server::app;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::router::auth::{Browser, with_json, without_body};
use crate::router::{Answer, SECRET, client_peer, request, send};
use crate::stack::{ApiStack, api};

/// The prefix of the internal admin API.
pub const ADMIN: &str = "/internal/admin/v1";
/// The acting admin's id, as the admin server names it.
pub const ADMIN_ID: &str = "0190f6a2-7c1e-7d3a-9b4e-00000000a001";
/// The acting admin's address.
pub const ADMIN_EMAIL: &str = "ops@example.org";
/// An id no row has.
pub const UNKNOWN: &str = "0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e";
/// The context the app attaches to a support request.
const CONTEXT: &str =
    r#"{"appVersion":"0.1.0","platform":"web","language":"en","screen":"/editor/:animationId"}"#;
/// The boundary between the parts of the forms sent.
const BOUNDARY: &str = "life-pixel-admin-test-boundary";
/// The entries of the audit log, oldest first.
const AUDIT_ENTRIES: &str = "select admin_id, admin_email, action, target_type, target_id, \
                             reason, before::text as before, after::text as after \
                             from audit_log order by id";
/// The admin whose entries [`fail_the_failing_admins_entries`] makes fail.
pub const FAILING_ADMIN: &str = "failing-admin";
const FAIL_ENTRIES: &str = "alter table audit_log add constraint audit_log_test_failure \
                            check (admin_id <> 'failing-admin')";
const ACCOUNT_ID: &str = "select id from accounts where email = $1::citext";

/// A request builder of the admin server, with the secret, acting for `(id, email)`.
pub fn acting((id, email): (&str, &str), method: Method, path: &str) -> Builder {
    request(method, &format!("{ADMIN}{path}"))
        .header(AUTHORIZATION, format!("Bearer {SECRET}"))
        .header("X-Admin-Id", id)
        .header("X-Admin-Email", email)
}

/// A request builder of the admin server, acting for [`ADMIN_ID`].
pub fn admin(method: Method, path: &str) -> Builder {
    acting((ADMIN_ID, ADMIN_EMAIL), method, path)
}

/// `GET <ADMIN><path>`, acting for [`ADMIN_ID`].
pub fn admin_get(path: &str) -> Request<Body> {
    without_body(admin(Method::GET, path))
}

/// `method <ADMIN><path>` with `{reason}`, acting for [`ADMIN_ID`].
pub fn with_reason(method: Method, path: &str, reason: &str) -> Request<Body> {
    with_json(admin(method, path), &json!({ "reason": reason }))
}

/// The answer of the admin listener to `request`.
pub async fn call(stack: &ApiStack, request: Request<Body>) -> Answer {
    let router = app::admin_router(stack.state.clone()).layer(MockConnectInfo(client_peer()));
    send(router, request).await
}

/// The answer of the admin listener to `request`, checked to have `status`.
pub async fn expect(stack: &ApiStack, status: u16, request: Request<Body>) -> Answer {
    let answer = call(stack, request).await;
    assert_eq!(answer.status.as_u16(), status, "{:?}", answer.body);
    answer
}

/// Signs `email` up: its session, and its account's id.
pub async fn user(stack: &ApiStack, email: &str) -> (Browser, String) {
    let browser = stack.sign_up(email).await;
    let query = sqlx::query_scalar::<_, Uuid>(ACCOUNT_ID).bind(email);
    let id = query.fetch_one(stack.database.pool()).await.unwrap();
    (browser, id.to_string())
}

/// The `multipart/form-data` body of a support request.
fn form(parts: &[(&str, &str)], screenshot: Option<&[u8]>) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, text) in parts {
        let part = format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{text}\r\n"
        );
        body.extend_from_slice(part.as_bytes());
    }
    if let Some(png) = screenshot {
        let headers = format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"screenshot\"; \
             filename=\"upload\"\r\nContent-Type: image/png\r\n\r\n"
        );
        body.extend_from_slice(headers.as_bytes());
        body.extend_from_slice(png);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

/// Sends a support request of `browser` in `category`, saying `message`, with `screenshot`: its
/// id.
pub async fn support_request(
    stack: &ApiStack,
    browser: &Browser,
    (category, message, screenshot): (&str, &str, Option<&[u8]>),
) -> String {
    let parts = [
        ("category", category),
        ("message", message),
        ("context", CONTEXT),
    ];
    let builder = api(browser, Method::POST, "/support-requests");
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let body = Body::from(form(&parts, screenshot));
    let request = builder
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .unwrap();
    id(&stack.created(request).await)
}

/// The id an answer's body carries.
pub fn id(body: &Value) -> String {
    body["id"].as_str().unwrap().to_owned()
}

/// An entry of the audit log, as the table keeps it.
#[derive(Debug, sqlx::FromRow)]
pub struct AuditRow {
    /// The admin's id.
    pub admin_id: String,
    /// The admin's address.
    pub admin_email: String,
    /// What they did.
    pub action: String,
    /// The kind of target.
    pub target_type: String,
    /// The target's id.
    pub target_id: String,
    /// Why.
    pub reason: Option<String>,
    /// The state before, as JSON text.
    pub before: Option<String>,
    /// The state after, as JSON text.
    pub after: Option<String>,
}

impl AuditRow {
    /// Who did what to what: the admin's id and address, the action, the target's type and id.
    pub fn summary(&self) -> [&str; 5] {
        [
            &self.admin_id,
            &self.admin_email,
            &self.action,
            &self.target_type,
            &self.target_id,
        ]
    }

    /// The state before the action.
    pub fn before(&self) -> Value {
        self.before
            .as_deref()
            .map_or(Value::Null, |text| serde_json::from_str(text).unwrap())
    }

    /// The state after the action.
    pub fn after(&self) -> Value {
        self.after
            .as_deref()
            .map_or(Value::Null, |text| serde_json::from_str(text).unwrap())
    }
}

/// Every entry of the audit log, oldest first.
pub async fn audit_entries(stack: &ApiStack) -> Vec<AuditRow> {
    let query = sqlx::query_as::<_, AuditRow>(AUDIT_ENTRIES);
    query.fetch_all(stack.database.pool()).await.unwrap()
}

/// Makes every audit entry of [`FAILING_ADMIN`] fail, as a database refusing the write would:
/// the change written with it must roll back.
pub async fn fail_the_failing_admins_entries(stack: &ApiStack) {
    let statement = sqlx::query(FAIL_ENTRIES);
    statement.execute(stack.database.pool()).await.unwrap();
}
