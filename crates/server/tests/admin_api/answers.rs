//! What the team does to a request — its status and assignee, internal notes, replies —, each
//! audited in its transaction; a reply emailed through Mailpit, with the link to the thread.

use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::request::Builder;
use axum::http::{Method, Request};
use life_pixel_server::testing::{MailMessage, TestMailbox};
use serde_json::{Value, json};

use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, get};
use crate::{
    ADMIN_EMAIL, ADMIN_ID, FAILING_ADMIN, UNKNOWN, acting, admin, audit_entries, call, expect,
    fail_the_failing_admins_entries, support_request, user,
};

/// How long a test waits for an email to reach Mailpit.
const MAIL_WAIT: Duration = Duration::from_secs(15);
/// The origin of the links of the emails, `LP_PUBLIC_URL`.
const PUBLIC_URL: &str = "http://localhost:8460";
const MESSAGES: &str = "select count(*) from support_messages where request_id::text = $1";

/// `PATCH /support-requests/{id}` with `body`, from [`ADMIN_ID`].
fn patch(id: &str, body: &Value) -> Request<Body> {
    with_json(
        admin(Method::PATCH, &format!("/support-requests/{id}")),
        body,
    )
}

/// The request of `builder`, a message of the team saying `body`.
fn message(builder: Builder, body: &str, internal: bool) -> Request<Body> {
    with_json(builder, &json!({ "body": body, "internal": internal }))
}

/// `POST /support-requests/{id}/messages`, from [`ADMIN_ID`].
fn post(id: &str) -> Builder {
    admin(Method::POST, &format!("/support-requests/{id}/messages"))
}

/// The text of `key` in the repository's English catalogue.
fn english(key: &str) -> String {
    let path = format!("{}/../../i18n/en.json", env!("CARGO_MANIFEST_DIR"));
    let catalogue: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    catalogue[key].as_str().unwrap().to_owned()
}

/// The first email titled `subject` in `mailbox`, once it arrives.
async fn email_titled(mailbox: &TestMailbox, subject: &str) -> MailMessage {
    let deadline = Instant::now() + MAIL_WAIT;
    loop {
        let all = mailbox.messages().await.unwrap();
        if let Some(found) = all.into_iter().find(|message| message.subject == subject) {
            return found;
        }
        assert!(
            Instant::now() < deadline,
            "{subject:?} to {}",
            mailbox.address()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// A person signed up at `email`, and a bug report of theirs: its id.
async fn sent(stack: &ApiStack, email: &str) -> (Browser, String) {
    let (browser, _) = user(stack, email).await;
    let request = support_request(stack, &browser, ("bug", "It crashes", None)).await;
    (browser, request)
}

/// The request `id` after `PATCH` with `body`.
async fn changed(stack: &ApiStack, id: &str, body: &Value) -> Value {
    expect(stack, 200, patch(id, body)).await.json()
}

/// The request `id` as `browser`, its author, sees it.
async fn seen_by(stack: &ApiStack, browser: &Browser, id: &str) -> Value {
    let request = get(browser, &format!("/support-requests/{id}"));
    stack.expect(200, request).await.json()
}

/// Checks that `mailbox` got the reply email of the request `id`, with the link to its thread
/// and without the reply itself.
async fn check_reply_email(mailbox: &TestMailbox, id: &str) {
    let email = email_titled(mailbox, &english("email.support_reply.subject")).await;
    let link = format!("{PUBLIC_URL}/support/{id}");
    let body = english("email.support_reply.body").replace("{link}", &link);
    assert_eq!(email.text.replace("\r\n", "\n").trim_end(), body);
    assert!(!email.text.contains("Fixed in 0.2"));
}

#[tokio::test]
async fn a_change_of_status_and_assignee_answers_the_request_after_it() {
    let stack = ApiStack::new().await;
    let (_, request) = sent(&stack, "ada@example.org").await;

    let both = json!({ "status": "in_progress", "assignedTo": "admin-2" });
    let assigned = changed(&stack, &request, &both).await;
    assert_eq!(assigned["status"], "in_progress");
    assert_eq!(assigned["assignedTo"], "admin-2");
    assert!(assigned["ageSeconds"].as_i64().unwrap() >= 0);
    let unassigned = changed(&stack, &request, &json!({ "assignedTo": null })).await;
    assert_eq!(unassigned["assignedTo"], Value::Null);
    assert_eq!(unassigned["status"], "in_progress");
    let resolved = changed(&stack, &request, &json!({ "status": "resolved" })).await;
    assert!(resolved["resolvedAt"].is_string());
    let reopened = changed(&stack, &request, &json!({ "status": "in_progress" })).await;
    assert_eq!(reopened["resolvedAt"], Value::Null);
}

#[tokio::test]
async fn each_change_is_audited_with_the_state_before_and_after() {
    let stack = ApiStack::new().await;
    let (_, request) = sent(&stack, "ada@example.org").await;
    let both = json!({ "status": "in_progress", "assignedTo": "admin-2" });
    changed(&stack, &request, &both).await;
    changed(&stack, &request, &json!({ "assignedTo": null })).await;

    let entries = audit_entries(&stack).await;

    assert_eq!(entries.len(), 2);
    let summary = [
        ADMIN_ID,
        ADMIN_EMAIL,
        "support.update",
        "support_request",
        &request,
    ];
    assert_eq!(entries[0].summary(), summary);
    assert_eq!(entries[0].reason, None);
    let before = json!({
        "status": "new", "assignedTo": null, "firstResponseAt": null, "resolvedAt": null
    });
    assert_eq!(entries[0].before(), before);
    let after = json!({
        "status": "in_progress", "assignedTo": "admin-2", "firstResponseAt": null,
        "resolvedAt": null
    });
    assert_eq!(entries[0].after(), after);
    assert_eq!(entries[1].before(), after);
    assert_eq!(entries[1].after()["assignedTo"], Value::Null);
}

#[tokio::test]
async fn a_change_must_change_something_known_on_a_known_request() {
    let stack = ApiStack::new().await;
    let (_, request) = sent(&stack, "ada@example.org").await;
    for body in [
        json!({}),
        json!({ "status": "lost" }),
        json!({ "assignedTo": " " }),
    ] {
        let answer = call(&stack, patch(&request, &body)).await;
        answer.assert_problem(400, "request.malformed");
    }
    let unknown = call(&stack, patch(UNKNOWN, &json!({ "status": "closed" }))).await;
    unknown.assert_problem(404, "support.request_not_found");
    assert!(audit_entries(&stack).await.is_empty());
}

#[tokio::test]
async fn an_internal_note_stays_with_the_team_and_changes_nothing_else() {
    let stack = ApiStack::new().await;
    let (ada, request) = sent(&stack, "ada@example.org").await;

    let note = message(post(&request), "Known issue", true);
    let note = expect(&stack, 201, note).await.json();

    assert_eq!(note["author"], "team");
    assert_eq!(note["adminId"], ADMIN_ID);
    assert_eq!(note["body"], "Known issue");
    assert_eq!(note["internal"], true);
    let seen = seen_by(&stack, &ada, &request).await;
    assert_eq!(seen["status"], "new");
    assert_eq!(seen["messages"].as_array().unwrap().len(), 1);
    let entries = audit_entries(&stack).await;
    assert_eq!(entries[0].action, "support.note");
    assert_eq!(entries[0].before, None);
    let after = json!({ "messageId": note["id"], "internal": true });
    assert_eq!(entries[0].after(), after);
}

#[tokio::test]
async fn a_message_needs_a_body_and_a_known_request() {
    let stack = ApiStack::new().await;
    let (_, request) = sent(&stack, "ada@example.org").await;
    for internal in [true, false] {
        let empty = call(&stack, message(post(&request), "  ", internal)).await;
        let params = empty.assert_problem(422, "support.message_length");
        assert_eq!(params, json!({ "min": 1, "max": 5000 }));
        let unknown = call(&stack, message(post(UNKNOWN), "Hello", internal)).await;
        unknown.assert_problem(404, "support.request_not_found");
    }
    assert!(audit_entries(&stack).await.is_empty());
}

#[tokio::test]
async fn a_reply_waits_for_the_person_and_is_emailed_with_the_link_to_the_thread() {
    let stack = ApiStack::with_mailpit().await;
    let mailbox = TestMailbox::new().unwrap();
    let (ada, request) = sent(&stack, mailbox.address()).await;

    let reply = message(post(&request), "Fixed in 0.2", false);
    let reply = expect(&stack, 201, reply).await.json();

    assert_eq!(reply["internal"], false);
    let seen = seen_by(&stack, &ada, &request).await;
    assert_eq!(seen["status"], "waiting_for_user");
    assert_eq!(seen["messages"][1]["body"], "Fixed in 0.2");
    let thread = crate::admin_get(&format!("/support-requests/{request}"));
    let thread = expect(&stack, 200, thread).await.json();
    assert!(thread["request"]["firstResponseAt"].is_string());
    let entries = audit_entries(&stack).await;
    assert_eq!(entries[0].action, "support.reply");
    assert_eq!(entries[0].before()["status"], "new");
    assert_eq!(entries[0].after()["status"], "waiting_for_user");
    assert_eq!(entries[0].after()["messageId"], reply["id"]);
    assert!(!entries[0].after.as_deref().unwrap().contains("Fixed"));
    check_reply_email(&mailbox, &request).await;
}

#[tokio::test]
async fn a_failed_entry_leaves_neither_the_change_nor_the_message() {
    let stack = ApiStack::new().await;
    let (ada, request) = sent(&stack, "ada@example.org").await;
    fail_the_failing_admins_entries(&stack).await;
    let failing = |method: Method, path: &str| acting((FAILING_ADMIN, ADMIN_EMAIL), method, path);

    let path = format!("/support-requests/{request}");
    let change = with_json(
        failing(Method::PATCH, &path),
        &json!({ "status": "closed" }),
    );
    let change = call(&stack, change).await;
    change.assert_problem(503, "service.unavailable");
    let reply = message(
        failing(Method::POST, &format!("{path}/messages")),
        "Hi",
        false,
    );
    let reply = call(&stack, reply).await;
    reply.assert_problem(503, "service.unavailable");

    assert_eq!(seen_by(&stack, &ada, &request).await["status"], "new");
    assert_eq!(stack.count(MESSAGES, &request).await, 1);
    assert!(audit_entries(&stack).await.is_empty());
}
