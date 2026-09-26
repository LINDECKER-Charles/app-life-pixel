//! A person's requests: the page of summaries, a thread without the team's internal notes, and
//! a reply, refused on a closed request.

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::{Value, json};

use crate::router::auth::{Browser, with_json};
use crate::router::empty;
use crate::stack::{API, ApiStack, api, get};
use crate::{REQUESTS, UNKNOWN, execute, id, send_request};

/// A visible answer and an internal note of the team on the request `$1`.
const TEAM_MESSAGES: &str = "insert into support_messages \
     (id, request_id, author, admin_id, body, internal, created_at) values \
     (gen_random_uuid(), $1, 'team', 'admin-1', 'Could you try again?', false, now()), \
     (gen_random_uuid(), $1, 'team', 'admin-1', 'Internal: known issue', true, now())";
const WAITING: &str = "update support_requests set status = 'waiting_for_user' where id = $1";
const CLOSED: &str = "update support_requests set status = 'closed' where id = $1";

/// `browser`'s reply saying `body` on the request `id`.
fn reply(browser: &Browser, id: &str, body: &str) -> Request<Body> {
    let path = format!("{REQUESTS}/{id}/messages");
    with_json(api(browser, Method::POST, &path), &json!({ "body": body }))
}

/// The thread of the request `id`, as `browser` sees it.
async fn thread(stack: &ApiStack, browser: &Browser, id: &str) -> Value {
    let path = format!("{REQUESTS}/{id}");
    stack.expect(200, get(browser, &path)).await.json()
}

/// The ids of the items of `page`.
fn ids(page: &Value) -> Vec<String> {
    page["items"].as_array().unwrap().iter().map(id).collect()
}

#[tokio::test]
async fn the_list_shows_ones_own_requests_newest_first_by_pages() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    let mut sent = Vec::new();
    for message in ["First", "Second", "Third"] {
        sent.push(id(&send_request(&stack, &ada, message).await));
    }
    let hers = id(&send_request(&stack, &grace, "Mine").await);

    let first = stack
        .expect(200, get(&ada, &format!("{REQUESTS}?limit=2")))
        .await;
    let first = first.json();
    assert_eq!(ids(&first), [sent[2].clone(), sent[1].clone()]);
    assert!(first["items"][0].get("messages").is_none());
    assert_eq!(first["items"][0]["status"], "new");
    let cursor = first["nextCursor"].as_str().unwrap();
    let next = get(&ada, &format!("{REQUESTS}?limit=2&cursor={cursor}"));
    let next = stack.expect(200, next).await.json();
    assert_eq!(ids(&next), [sent[0].clone()]);
    assert_eq!(next["nextCursor"], Value::Null);
    let theirs = stack.expect(200, get(&grace, REQUESTS)).await.json();
    assert_eq!(ids(&theirs), [hers]);
}

#[tokio::test]
async fn a_thread_never_shows_the_teams_internal_notes() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let request = id(&send_request(&stack, &ada, "It crashes").await);
    execute(&stack, TEAM_MESSAGES, &request).await;

    let shown = thread(&stack, &ada, &request).await;

    let messages = shown["messages"].as_array().unwrap();
    let authors: Vec<&Value> = messages.iter().map(|message| &message["author"]).collect();
    assert_eq!(authors, [&json!("user"), &json!("team")]);
    assert_eq!(messages[0]["body"], "It crashes");
    assert_eq!(messages[1]["body"], "Could you try again?");
    assert!(!shown.to_string().contains("Internal"));
}

#[tokio::test]
async fn a_request_is_its_owners_alone() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    let request = id(&send_request(&stack, &ada, "Private").await);

    for other in [request.as_str(), UNKNOWN] {
        let path = format!("{REQUESTS}/{other}");
        let refused = get(&grace, &path);
        stack
            .refused(refused, (404, "support.request_not_found"))
            .await;
        let refused = reply(&grace, other, "Hello");
        stack
            .refused(refused, (404, "support.request_not_found"))
            .await;
    }
    let malformed = get(&ada, &format!("{REQUESTS}/not-an-id"));
    stack.refused(malformed, (400, "request.malformed")).await;
    let anonymous = empty(Method::GET, &format!("{API}{REQUESTS}"));
    stack
        .refused(anonymous, (401, "auth.unauthenticated"))
        .await;
}

#[tokio::test]
async fn a_reply_brings_a_request_waiting_for_its_owner_back_in_progress() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let request = id(&send_request(&stack, &ada, "Blank canvas").await);
    execute(&stack, WAITING, &request).await;

    let message = stack.created(reply(&ada, &request, " Still blank ")).await;

    assert_eq!(message["author"], "user");
    assert_eq!(message["body"], "Still blank");
    let shown = thread(&stack, &ada, &request).await;
    assert_eq!(shown["status"], "in_progress");
    assert_eq!(shown["messages"].as_array().unwrap().len(), 2);
    assert_eq!(shown["messages"][1], message);
    stack.created(reply(&ada, &request, "And again")).await;
    assert_eq!(
        thread(&stack, &ada, &request).await["status"],
        "in_progress"
    );
}

#[tokio::test]
async fn a_closed_request_or_an_empty_reply_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let request = id(&send_request(&stack, &ada, "Done").await);

    let empty_reply = reply(&ada, &request, "  ");
    stack
        .refused(empty_reply, (422, "support.message_length"))
        .await;
    execute(&stack, CLOSED, &request).await;
    let refused = reply(&ada, &request, "One more thing");

    stack
        .refused(refused, (409, "support.request_closed"))
        .await;
    let shown = thread(&stack, &ada, &request).await;
    assert_eq!(shown["messages"].as_array().unwrap().len(), 1);
}
