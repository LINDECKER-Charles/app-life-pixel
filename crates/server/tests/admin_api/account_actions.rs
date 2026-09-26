//! What an admin does to an account — suspension, reactivation, data export, erasure —, each
//! with its entry in the audit log, written in the change's transaction: a failed entry leaves
//! the account as it was.

use axum::http::Method;
use serde_json::json;

use crate::router::auth::Browser;
use crate::stack::{ApiStack, get};
use crate::{
    ADMIN_EMAIL, ADMIN_ID, FAILING_ADMIN, UNKNOWN, acting, audit_entries, call, expect,
    fail_the_failing_admins_entries, support_request, user, with_reason,
};

const STATUS: &str = "select count(*) from accounts where status = 'suspended' and id::text = $1";
const SESSIONS: &str = "select count(*) from sessions where account_id::text = $1";
const ACCOUNTS: &str = "select count(*) from accounts where id::text = $1";
const AUDITED_EMAILS: &str = "select count(*) from audit_log \
                              where concat_ws(' ', target_id, reason, before, after) like $1";

/// Whether `browser`'s session still works.
async fn is_signed_in(stack: &ApiStack, browser: &Browser) -> bool {
    stack.send(get(browser, "/auth/session")).await.status == 200
}

#[tokio::test]
async fn a_suspension_ends_the_sessions_and_a_reactivation_undoes_it_both_audited() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;

    let suspend = with_reason(Method::POST, &format!("/users/{ada}/suspend"), " Spam ");
    expect(&stack, 204, suspend).await;
    assert_eq!(stack.count(STATUS, &ada).await, 1);
    assert_eq!(stack.count(SESSIONS, &ada).await, 0);
    assert!(!is_signed_in(&stack, &browser).await);
    let reactivate = with_reason(Method::POST, &format!("/users/{ada}/reactivate"), "Appeal");
    expect(&stack, 204, reactivate).await;
    assert_eq!(stack.count(STATUS, &ada).await, 0);

    let entries = audit_entries(&stack).await;
    assert_eq!(entries.len(), 2);
    let (suspension, reactivation) = (&entries[0], &entries[1]);
    let summary = [ADMIN_ID, ADMIN_EMAIL, "user.suspend", "user", &ada];
    assert_eq!(suspension.summary(), summary);
    assert_eq!(suspension.reason.as_deref(), Some("Spam"));
    assert_eq!(suspension.before(), json!({ "status": "active" }));
    assert_eq!(suspension.after(), json!({ "status": "suspended" }));
    assert_eq!(reactivation.action, "user.reactivate");
    assert_eq!(reactivation.after(), json!({ "status": "active" }));
}

#[tokio::test]
async fn an_action_on_an_account_needs_a_reason_and_an_account() {
    let stack = ApiStack::new().await;
    let (_, ada) = user(&stack, "ada@example.org").await;
    for (method, action) in [
        (Method::POST, "/suspend"),
        (Method::POST, "/reactivate"),
        (Method::POST, "/export"),
        (Method::DELETE, ""),
    ] {
        let blank = with_reason(method.clone(), &format!("/users/{ada}{action}"), "  ");
        let params = call(&stack, blank)
            .await
            .assert_problem(422, "admin.reason_length");
        assert_eq!(params, json!({ "min": 1, "max": 1000 }));
        let long = with_reason(
            method.clone(),
            &format!("/users/{ada}{action}"),
            &"a".repeat(1001),
        );
        call(&stack, long)
            .await
            .assert_problem(422, "admin.reason_length");
        let unknown = with_reason(method, &format!("/users/{UNKNOWN}{action}"), "Spam");
        call(&stack, unknown)
            .await
            .assert_problem(404, "admin.user_not_found");
    }
    assert_eq!(stack.count(ACCOUNTS, &ada).await, 1);
    assert!(audit_entries(&stack).await.is_empty());
}

#[tokio::test]
async fn a_failed_entry_leaves_the_account_active_and_its_sessions_open() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;
    fail_the_failing_admins_entries(&stack).await;

    let builder = acting(
        (FAILING_ADMIN, ADMIN_EMAIL),
        Method::POST,
        &format!("/users/{ada}/suspend"),
    );
    let suspend = crate::router::auth::with_json(builder, &json!({ "reason": "Spam" }));
    call(&stack, suspend)
        .await
        .assert_problem(503, "service.unavailable");

    assert_eq!(stack.count(STATUS, &ada).await, 0);
    assert_eq!(stack.count(SESSIONS, &ada).await, 1);
    assert!(is_signed_in(&stack, &browser).await);
    assert!(audit_entries(&stack).await.is_empty());
}

#[tokio::test]
async fn the_export_is_the_owners_zip_and_its_read_is_audited_with_its_reason() {
    let stack = ApiStack::new().await;
    let (_, ada) = user(&stack, "ada@example.org").await;

    let export = with_reason(
        Method::POST,
        &format!("/users/{ada}/export"),
        " Access request ",
    );
    let answer = expect(&stack, 200, export).await;

    assert_eq!(answer.header("content-type"), "application/zip");
    assert!(
        answer
            .header("content-disposition")
            .starts_with("attachment; filename=\"life-pixel-export-")
    );
    assert!(answer.body.starts_with(b"PK"));
    let entries = audit_entries(&stack).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "user.export");
    assert_eq!(entries[0].target_id, ada);
    assert_eq!(entries[0].reason.as_deref(), Some("Access request"));
    assert_eq!(
        (entries[0].before.as_ref(), entries[0].after.as_ref()),
        (None, None)
    );
}

#[tokio::test]
async fn an_erasure_deletes_the_account_and_keeps_its_id_and_the_reason_never_its_address() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;
    support_request(&stack, &browser, ("data_protection", "Erase me", None)).await;

    let erase = with_reason(
        Method::DELETE,
        &format!("/users/{ada}"),
        "Asked for erasure",
    );
    expect(&stack, 204, erase).await;

    assert_eq!(stack.count(ACCOUNTS, &ada).await, 0);
    assert!(!is_signed_in(&stack, &browser).await);
    let entries = audit_entries(&stack).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "user.delete");
    assert_eq!(entries[0].target_id, ada);
    assert_eq!(entries[0].reason.as_deref(), Some("Asked for erasure"));
    assert_eq!(entries[0].before()["status"], "active");
    assert_eq!(entries[0].after, None);
    assert_eq!(stack.count(AUDITED_EMAILS, "%ada@example.org%").await, 0);
    let again = with_reason(Method::DELETE, &format!("/users/{ada}"), "Twice");
    call(&stack, again)
        .await
        .assert_problem(404, "admin.user_not_found");
}

#[tokio::test]
async fn a_failed_entry_leaves_the_account_in_place() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;
    fail_the_failing_admins_entries(&stack).await;

    let builder = acting(
        (FAILING_ADMIN, ADMIN_EMAIL),
        Method::DELETE,
        &format!("/users/{ada}"),
    );
    let erase = crate::router::auth::with_json(builder, &json!({ "reason": "Spam" }));
    call(&stack, erase)
        .await
        .assert_problem(503, "service.unavailable");

    assert_eq!(stack.count(ACCOUNTS, &ada).await, 1);
    assert!(is_signed_in(&stack, &browser).await);
    assert!(audit_entries(&stack).await.is_empty());
}
