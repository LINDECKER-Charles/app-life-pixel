//! Who may call `/mcp`: a valid token of an active account, within the `mcp` rate limit of the
//! token, for the tools its scopes grant, on its account's library only.

use serde_json::{Value, json};
use time::Duration;

use crate::stack::{ApiStack, delete, get};
use crate::tokens::{
    create_token, list_tools, result, rpc, secret, tool_call, tool_failure, tool_output,
};
use crate::{animation, clock_at, listed_titles, stack_at};

const SUSPEND: &str = "update accounts set status = 'suspended' where email = $1";

/// Checks that `answer` refuses the token with `401 token.invalid` and a bearer challenge.
fn assert_invalid(answer: &crate::router::Answer) {
    answer.assert_problem(401, "token.invalid");
    assert_eq!(answer.header("www-authenticate"), "Bearer");
}

#[tokio::test]
async fn a_call_without_a_valid_token_is_refused_with_a_bearer_challenge() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let created = create_token(&stack, &ada, ("Laptop", &["read"], 30)).await;
    let revoked = format!("Bearer {}", created["token"].as_str().unwrap());
    let path = format!("/tokens/{}", created["id"].as_str().unwrap());
    stack.expect(204, delete(&ada, &path)).await;
    let unknown = format!("Bearer lp_pat_{}", "A".repeat(43));
    let authorizations = [
        None,
        Some("Basic YWRhOnNlY3JldA=="),
        Some("Bearer"),
        Some("Bearer not-a-token"),
        Some(unknown.as_str()),
        Some(revoked.as_str()),
    ];
    for authorization in authorizations {
        let answer = stack
            .send(rpc(authorization, ("tools/list", json!({}))))
            .await;
        assert_invalid(&answer);
    }
}

#[tokio::test]
async fn an_expired_token_is_refused() {
    let clock = clock_at(time::OffsetDateTime::now_utc());
    let stack = stack_at(&clock, |_| {}).await;
    let ada = stack.sign_up("ada@example.org").await;
    let created = create_token(&stack, &ada, ("Laptop", &["read"], 30)).await;
    let secret = created["token"].as_str().unwrap();

    clock.advance(Duration::days(30) - Duration::minutes(1));
    result(&stack.send(list_tools(secret)).await);
    clock.advance(Duration::minutes(1));
    assert_invalid(&stack.send(list_tools(secret)).await);
}

#[tokio::test]
async fn the_token_of_a_suspended_account_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let secret = secret(&stack, &ada, &["read"]).await;
    let suspend = sqlx::query(SUSPEND).bind("ada@example.org");
    suspend.execute(stack.database.pool()).await.unwrap();

    let answer = stack
        .send(tool_call(&secret, ("list_animations", json!({}))))
        .await;

    answer.assert_problem(403, "auth.account_suspended");
}

/// The arguments of `create_animation` for a new `Walk` in the project `Game`.
fn walk() -> Value {
    json!({ "title": "Walk", "width": 16, "height": 16, "project_name": "Game" })
}

#[tokio::test]
async fn a_token_calls_the_tools_its_scopes_grant() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let mascot = animation(&stack, &ada, "Mascot").await;
    let reader = secret(&stack, &ada, &["read"]).await;

    let tools = result(&stack.send(list_tools(&reader)).await);
    assert_eq!(tools["tools"].as_array().unwrap().len(), 11);
    let export = json!({ "id": mascot["id"], "format": "gif" });
    let refusals = [
        (("create_animation", walk()), "write"),
        (("export", export), "export"),
    ];
    for (call, required) in refusals {
        let failure = tool_failure(&stack.send(tool_call(&reader, call)).await);
        let expected = json!({ "code": "token.scope", "params": { "required": required } });
        assert_eq!(failure, expected);
    }
    let writer = secret(&stack, &ada, &["write"]).await;
    let created = stack.send(tool_call(&writer, ("create_animation", walk())));
    assert_eq!(tool_output(&created.await)["title"], "Walk");
}

#[tokio::test]
async fn a_token_reaches_its_account_library_only() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    animation(&stack, &ada, "Mascot").await;
    let hero = animation(&stack, &grace, "Hero").await;
    let reader = secret(&stack, &ada, &["read"]).await;

    assert_eq!(listed_titles(&stack, &reader).await, ["Mascot"]);
    let hidden = tool_call(&reader, ("get_animation", json!({ "id": hero["id"] })));
    let hidden = stack.send(hidden).await;
    assert_eq!(tool_failure(&hidden)["code"], "library.animation_not_found");
}

#[tokio::test]
async fn a_call_records_its_use_and_its_event() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let reader = secret(&stack, &ada, &["read"]).await;

    listed_titles(&stack, &reader).await;
    let create = json!({ "title": "Walk", "width": 16, "height": 16, "project_name": "Game" });
    stack
        .send(tool_call(&reader, ("create_animation", create)))
        .await;

    let tokens = stack.expect(200, get(&ada, "/tokens")).await.json();
    assert!(tokens[0]["lastUsedAt"].is_string(), "{tokens}");
    let calls: Vec<Value> = stack
        .events
        .events()
        .into_iter()
        .filter(|event| event.name == "mcp_tool_called")
        .map(|event| json!(event.properties))
        .collect();
    let expected = json!([
        [["tool", "list_animations"], ["outcome", "ok"]],
        [["tool", "create_animation"], ["outcome", "error"]],
    ]);
    assert_eq!(json!(calls), expected);
}

#[tokio::test]
async fn the_mcp_rate_limit_counts_by_token() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let first = secret(&stack, &ada, &["read"]).await;
    let second = secret(&stack, &ada, &["read"]).await;
    // One call comes back every half second: a slow run may get a few beyond the burst.
    let mut accepted = 0;
    let refused = loop {
        let answer = stack.send(list_tools(&first)).await;
        if answer.status != 200 || accepted == 200 {
            break answer;
        }
        accepted += 1;
    };

    assert!(accepted >= 120, "{accepted}");
    let params = refused.assert_problem(429, "rate_limit.exceeded");
    assert!(params["retryAfterSeconds"].as_u64().unwrap() >= 1);
    assert!(refused.headers.contains_key("retry-after"));
    result(&stack.send(list_tools(&second)).await);
}
