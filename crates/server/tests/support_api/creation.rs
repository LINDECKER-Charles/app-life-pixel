//! `POST /support-requests`: a request, its first message and its context, and what the route
//! refuses.

use axum::http::Method;
use axum::http::header::CONTENT_TYPE;
use life_pixel_core::limits::SUPPORT_MESSAGE_MAX_CHARS;
use serde_json::{Value, json};

use crate::router::auth::{from_app, with_json};
use crate::stack::{API, ApiStack, api};
use crate::{CONTEXT, Part, REQUESTS, id, new_request, send_request, stored_requests, with_form};

const CONTEXT_OF: &str = "select context::text from support_requests where id = $1::uuid";

#[tokio::test]
async fn a_request_starts_new_with_its_message_first_and_keeps_its_context() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    let created = send_request(&stack, &ada, "  The canvas stays blank.  ").await;

    assert_eq!(created["category"], "bug");
    assert_eq!(created["status"], "new");
    assert_eq!(created["hasScreenshot"], false);
    assert_eq!(created["createdAt"], created["updatedAt"]);
    let messages = created["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["author"], "user");
    assert_eq!(messages[0]["body"], "The canvas stays blank.");
    let mut fields: Vec<&String> = created.as_object().unwrap().keys().collect();
    fields.sort();
    let expected = [
        "category",
        "createdAt",
        "hasScreenshot",
        "id",
        "messages",
        "status",
        "updatedAt",
    ];
    assert_eq!(fields, expected);
    let stored: String = sqlx::query_scalar(CONTEXT_OF)
        .bind(id(&created))
        .fetch_one(stack.database.pool())
        .await
        .unwrap();
    let stored: Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(stored, serde_json::from_str::<Value>(CONTEXT).unwrap());
}

#[tokio::test]
async fn a_request_records_its_category() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    let request = new_request(&ada, ("data_protection", "Erase my data"), None);
    stack.created(request).await;

    let events = stack.events.events();
    let created: Vec<_> = events
        .iter()
        .filter(|event| event.name == "support_request_created")
        .collect();
    assert_eq!(created.len(), 1);
    let properties = &created[0].properties;
    assert_eq!(
        properties,
        &vec![("category", "data_protection".to_owned())]
    );
}

#[tokio::test]
async fn a_category_or_a_message_outside_the_rules_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let too_long = "a".repeat(SUPPORT_MESSAGE_MAX_CHARS + 1);

    let praise = new_request(&ada, ("praise", "Great app"), None);
    stack.refused(praise, (422, "support.category")).await;
    for message in ["   ", too_long.as_str()] {
        let refused = new_request(&ada, ("bug", message), None);
        let params = stack
            .refused(refused, (422, "support.message_length"))
            .await;
        assert_eq!(
            params,
            json!({ "min": 1, "max": SUPPORT_MESSAGE_MAX_CHARS })
        );
    }
    assert_eq!(stored_requests(&stack).await, 0);
}

#[tokio::test]
async fn a_form_missing_a_part_or_with_a_context_that_is_not_the_apps_is_malformed() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let with_ids = CONTEXT.replace(":animationId", "0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e");
    let unknown = ("url", Part::Text("https://example.org"));
    let endings: [&[(&str, Part<'_>)]; 3] = [
        &[],
        &[("context", Part::Text(&with_ids))],
        &[("context", Part::Text(CONTEXT)), unknown],
    ];

    for ending in endings {
        let mut parts = vec![
            ("category", Part::Text("bug")),
            ("message", Part::Text("Hi")),
        ];
        parts.extend_from_slice(ending);
        let request = with_form(api(&ada, Method::POST, REQUESTS), &parts);
        stack.refused(request, (400, "request.malformed")).await;
    }
}

#[tokio::test]
async fn a_request_is_a_form_never_json() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let body = json!({ "category": "bug", "message": "Hi" });

    let as_json = with_json(api(&ada, Method::POST, REQUESTS), &body);

    stack
        .refused(as_json, (415, "request.unsupported_media_type"))
        .await;
}

#[tokio::test]
async fn a_request_needs_a_session_and_its_token() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let path = format!("{API}{REQUESTS}");

    let anonymous = with_form(from_app(Method::POST, &path), &[]);
    stack
        .refused(anonymous, (401, "auth.unauthenticated"))
        .await;
    let tokenless = ada.cookie(from_app(Method::POST, &path));
    let tokenless = tokenless
        .header(CONTENT_TYPE, "multipart/form-data; boundary=x")
        .body(axum::body::Body::empty())
        .unwrap();
    stack.refused(tokenless, (403, "auth.csrf")).await;
}

#[tokio::test]
async fn an_account_sends_at_most_ten_requests_a_day() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    for index in 0..10 {
        send_request(&stack, &ada, &format!("Request {index}")).await;
    }
    let refused = stack
        .send(new_request(&ada, ("bug", "One more"), None))
        .await;

    let params = refused.assert_problem(429, "rate_limit.exceeded");
    assert!(params["retryAfterSeconds"].as_u64().unwrap() > 0);
    let grace = stack.sign_up("grace@example.org").await;
    send_request(&stack, &grace, "Mine").await;
}
