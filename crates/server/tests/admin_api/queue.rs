//! The support queue: the requests with their account's address and age, filtered and by pages;
//! a request's thread with the team's internal notes and the context; its screenshot.

use life_pixel_service::support::memory::png_screenshot_with_text;
use serde_json::Value;
use uuid::Uuid;

use crate::stack::ApiStack;
use crate::{UNKNOWN, admin_get, call, expect, support_request, user};

/// An internal note of the team on the request `$1`.
const NOTE: &str = "insert into support_messages \
     (id, request_id, author, admin_id, body, internal, created_at) values \
     (gen_random_uuid(), $1, 'team', 'admin-1', 'Known issue', true, now())";
const ASSIGN: &str = "update support_requests set assigned_to = 'admin-2', \
                      status = 'in_progress' where id = $1";
const AGE: &str = "update support_requests set created_at = now() - interval '2 hours' \
                   where id = $1";

/// Runs `statement` with the request `id` as its `$1`.
async fn execute(stack: &ApiStack, statement: &'static str, id: &str) {
    let query = sqlx::query(statement).bind(Uuid::parse_str(id).unwrap());
    query.execute(stack.database.pool()).await.unwrap();
}

/// The ids of the items of `page`.
fn ids(page: &Value) -> Vec<&str> {
    let items = page["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect()
}

/// The page of `GET /support-requests<query>`.
async fn queue(stack: &ApiStack, query: &str) -> Value {
    expect(stack, 200, admin_get(&format!("/support-requests{query}")))
        .await
        .json()
}

/// Three requests of two accounts, the second assigned, the first sent two hours ago: the
/// stack, the second account's id, and the requests' ids, oldest first.
async fn three_requests() -> (ApiStack, String, [String; 3]) {
    let stack = ApiStack::new().await;
    let (ada, _) = user(&stack, "ada@example.org").await;
    let (grace, grace_id) = user(&stack, "grace@example.org").await;
    let bug = support_request(&stack, &ada, ("bug", "It crashes", None)).await;
    let billing = support_request(&stack, &grace, ("billing", "Invoice", None)).await;
    let other = support_request(&stack, &grace, ("other", "Hello", None)).await;
    execute(&stack, ASSIGN, &billing).await;
    execute(&stack, AGE, &bug).await;
    (stack, grace_id, [bug, billing, other])
}

#[tokio::test]
async fn the_queue_lists_every_account_s_requests_with_their_address_and_age() {
    let (stack, grace_id, [bug, billing, other]) = three_requests().await;

    let all = queue(&stack, "").await;

    assert_eq!(ids(&all), [other.as_str(), billing.as_str(), bug.as_str()]);
    let assigned = &all["items"][1];
    assert_eq!(assigned["email"], "grace@example.org");
    assert_eq!(assigned["accountId"], grace_id);
    assert_eq!(assigned["category"], "billing");
    assert_eq!(assigned["status"], "in_progress");
    assert_eq!(assigned["assignedTo"], "admin-2");
    assert_eq!(assigned["firstResponseAt"], Value::Null);
    assert_eq!(assigned["hasScreenshot"], false);
    assert!(all["items"][2]["ageSeconds"].as_i64().unwrap() >= 7200);
}

#[tokio::test]
async fn the_queue_filters_by_status_category_and_assignee_by_pages() {
    let (stack, _, [bug, billing, other]) = three_requests().await;

    let new = queue(&stack, "?status=new").await;
    assert_eq!(ids(&new), [other.as_str(), bug.as_str()]);
    assert_eq!(ids(&queue(&stack, "?category=bug").await), [bug.as_str()]);
    let assigned = queue(&stack, "?assignee=admin-2").await;
    assert_eq!(ids(&assigned), [billing.as_str()]);
    let page = queue(&stack, "?limit=2").await;
    let cursor = page["nextCursor"].as_str().unwrap();
    let next = queue(&stack, &format!("?limit=2&cursor={cursor}")).await;
    assert_eq!(ids(&next), [bug.as_str()]);
}

#[tokio::test]
async fn the_queue_refuses_an_unknown_status_or_category() {
    let stack = ApiStack::new().await;
    for query in ["?status=lost", "?category=praise", "?cursor=nope"] {
        let answer = call(&stack, admin_get(&format!("/support-requests{query}"))).await;
        answer.assert_problem(400, "request.malformed");
    }
}

/// Checks that `message` is by `author`, internal or not, and by the admin `admin_id`.
fn check_message(message: &Value, (author, internal): (&str, bool), admin_id: &Value) {
    assert_eq!(message["author"], author);
    assert_eq!(message["internal"], internal);
    assert_eq!(&message["adminId"], admin_id);
}

#[tokio::test]
async fn a_thread_shows_the_internal_notes_and_the_context() {
    let stack = ApiStack::new().await;
    let (ada, _) = user(&stack, "ada@example.org").await;
    let request = support_request(&stack, &ada, ("bug", "It crashes", None)).await;
    execute(&stack, NOTE, &request).await;

    let thread = admin_get(&format!("/support-requests/{request}"));
    let thread = expect(&stack, 200, thread).await.json();

    assert_eq!(thread["request"]["id"], request);
    assert_eq!(thread["request"]["email"], "ada@example.org");
    assert_eq!(thread["hasScreenshot"], false);
    assert_eq!(thread["context"]["appVersion"], "0.1.0");
    assert_eq!(thread["context"]["screen"], "/editor/:animationId");
    let messages = thread["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    check_message(&messages[0], ("user", false), &Value::Null);
    assert_eq!(messages[0]["body"], "It crashes");
    check_message(&messages[1], ("team", true), &Value::from("admin-1"));
    let unknown = call(&stack, admin_get(&format!("/support-requests/{UNKNOWN}"))).await;
    unknown.assert_problem(404, "support.request_not_found");
}

#[tokio::test]
async fn the_screenshot_is_the_stored_png() {
    let stack = ApiStack::new().await;
    let (ada, _) = user(&stack, "ada@example.org").await;
    let png = png_screenshot_with_text((8, 6), ("Comment", "kept out"));
    let with = support_request(&stack, &ada, ("bug", "Look", Some(&png))).await;
    let without = support_request(&stack, &ada, ("bug", "No image", None)).await;

    let answer = expect(
        &stack,
        200,
        admin_get(&format!("/support-requests/{with}/screenshot")),
    )
    .await;
    assert_eq!(answer.header("content-type"), "image/png");
    assert!(answer.body.starts_with(b"\x89PNG\r\n\x1a\n"));
    let thread = expect(&stack, 200, admin_get(&format!("/support-requests/{with}"))).await;
    assert_eq!(thread.json()["hasScreenshot"], true);
    let none = call(
        &stack,
        admin_get(&format!("/support-requests/{without}/screenshot")),
    )
    .await;
    none.assert_problem(404, "admin.screenshot_not_found");
    let unknown = call(
        &stack,
        admin_get(&format!("/support-requests/{UNKNOWN}/screenshot")),
    )
    .await;
    unknown.assert_problem(404, "support.request_not_found");
}
