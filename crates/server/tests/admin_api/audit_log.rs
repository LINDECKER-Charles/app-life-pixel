//! `GET /audit-log`: the entries newest first, filtered and by pages; and the table itself,
//! which refuses every update, delete and truncate.

use axum::http::Method;
use serde_json::Value;

use crate::stack::ApiStack;
use crate::{ADMIN_ID, acting, admin_get, call, expect, user, with_reason};

const UPDATE: &str = "update audit_log set reason = 'rewritten'";
const DELETE: &str = "delete from audit_log";
const TRUNCATE: &str = "truncate audit_log";
const ENTRIES: &str = "select count(*) from audit_log where reason <> $1";

/// The actions of the items of `page`.
fn actions(page: &Value) -> Vec<&str> {
    let items = page["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["action"].as_str().unwrap())
        .collect()
}

/// The page of `GET /audit-log<query>`.
async fn log(stack: &ApiStack, query: &str) -> Value {
    expect(stack, 200, admin_get(&format!("/audit-log{query}")))
        .await
        .json()
}

/// Suspends, then reactivates, `ada` as [`ADMIN_ID`]; then suspends `grace` as another admin.
async fn three_entries(stack: &ApiStack, (ada, grace): (&str, &str)) {
    for (path, reason) in [("suspend", "Spam"), ("reactivate", "Appeal")] {
        let action = with_reason(Method::POST, &format!("/users/{ada}/{path}"), reason);
        expect(stack, 204, action).await;
    }
    let other = acting(
        ("admin-2", "second@example.org"),
        Method::POST,
        &format!("/users/{grace}/suspend"),
    );
    let other = crate::router::auth::with_json(other, &serde_json::json!({ "reason": "Abuse" }));
    expect(stack, 204, other).await;
}

/// Two accounts, and [`three_entries`] about them: the stack and the accounts' ids.
async fn logged() -> (ApiStack, String, String) {
    let stack = ApiStack::new().await;
    let (_, ada) = user(&stack, "ada@example.org").await;
    let (_, grace) = user(&stack, "grace@example.org").await;
    three_entries(&stack, (&ada, &grace)).await;
    (stack, ada, grace)
}

#[tokio::test]
async fn the_log_lists_the_entries_newest_first() {
    let (stack, _, grace) = logged().await;

    let all = log(&stack, "").await;

    assert_eq!(
        actions(&all),
        ["user.suspend", "user.reactivate", "user.suspend"]
    );
    let newest = &all["items"][0];
    assert_eq!(newest["adminId"], "admin-2");
    assert_eq!(newest["adminEmail"], "second@example.org");
    assert_eq!(newest["targetType"], "user");
    assert_eq!(newest["targetId"], grace);
    assert_eq!(newest["reason"], "Abuse");
    assert_eq!(newest["before"]["status"], "active");
    assert_eq!(newest["after"]["status"], "suspended");
    assert!(newest["at"].is_string());
    let items = all["items"].as_array().unwrap();
    let ids: Vec<i64> = items
        .iter()
        .map(|item| item["id"].as_i64().unwrap())
        .collect();
    assert!(ids.is_sorted_by(|a, b| a > b));
}

#[tokio::test]
async fn the_log_filters_by_admin_action_and_target() {
    let (stack, ada, _) = logged().await;

    let mine = log(&stack, &format!("?adminId={ADMIN_ID}")).await;
    assert_eq!(actions(&mine), ["user.reactivate", "user.suspend"]);
    let suspensions = log(&stack, "?action=user.suspend").await;
    assert_eq!(suspensions["items"].as_array().unwrap().len(), 2);
    let about_ada = log(&stack, &format!("?targetId={ada}")).await;
    assert_eq!(actions(&about_ada), ["user.reactivate", "user.suspend"]);
    let nobody = log(&stack, "?adminId=nobody").await;
    assert_eq!(nobody["items"], serde_json::json!([]));
}

#[tokio::test]
async fn the_log_reads_by_pages() {
    let (stack, ada, _) = logged().await;

    let first = log(&stack, "?limit=2").await;

    assert_eq!(actions(&first), ["user.suspend", "user.reactivate"]);
    let cursor = first["nextCursor"].as_str().unwrap();
    let next = log(&stack, &format!("?limit=2&cursor={cursor}")).await;
    assert_eq!(actions(&next), ["user.suspend"]);
    assert_eq!(next["items"][0]["targetId"], ada);
    assert_eq!(next["nextCursor"], Value::Null);
}

#[tokio::test]
async fn the_log_refuses_a_cursor_that_does_not_parse() {
    let stack = ApiStack::new().await;
    let answer = call(&stack, admin_get("/audit-log?cursor=nope")).await;
    answer.assert_problem(400, "request.malformed");
}

#[tokio::test]
async fn the_table_refuses_every_update_delete_and_truncate() {
    let (stack, _, _) = logged().await;

    for statement in [UPDATE, DELETE, TRUNCATE] {
        let refused = sqlx::query(statement).execute(stack.database.pool()).await;
        let error = refused.unwrap_err().to_string();
        assert!(
            error.contains("audit_log is append-only"),
            "{statement}: {error}"
        );
    }
    assert_eq!(stack.count(ENTRIES, "rewritten").await, 3);
}
