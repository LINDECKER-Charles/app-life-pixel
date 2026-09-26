//! `POST /tokens`: the answer, the checks of the name, scopes and lifetime, and the limit of
//! active tokens.

use axum::http::Method;
use serde_json::{Value, json};

use crate::TOKENS;
use crate::router::auth::{from_app, with_json};
use crate::stack::{ApiStack, api, delete};
use crate::tokens::create_token;

/// The account's token count, active or not.
const TOKEN_COUNT: &str = "select count(*) from access_tokens t join accounts a \
                           on a.id = t.account_id where a.email = $1";
/// The hash kept for a token: never its secret.
const HASH_OF: &str = "select count(*) from access_tokens \
                       where token_hash = sha256(convert_to($1, 'UTF8'))";

#[tokio::test]
async fn a_new_token_is_shown_once_with_the_server_to_register_it_on() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    let created = create_token(&stack, &ada, ("  Claude Code  ", &["write", "read"], 90)).await;

    let secret = created["token"].as_str().unwrap();
    assert!(secret.starts_with("lp_pat_"), "{secret}");
    assert_eq!(secret.len(), "lp_pat_".len() + 43);
    assert_eq!(created["prefix"], secret[..11]);
    assert_eq!(created["name"], "Claude Code");
    assert_eq!(created["scopes"], json!(["read", "write"]));
    assert_eq!(created["lastUsedAt"], Value::Null);
    let mcp = json!({ "serverName": "life-pixel-dev", "url": "http://localhost:8460/mcp" });
    assert_eq!(created["mcp"], mcp);
    let created_at = time_of(&created["createdAt"]);
    let expires_at = time_of(&created["expiresAt"]);
    assert_eq!(expires_at - created_at, time::Duration::days(90));
    assert_eq!(stack.count(HASH_OF, secret).await, 1);
    assert_eq!(stack.count(TOKEN_COUNT, "ada@example.org").await, 1);
}

/// The instant of an RFC 3339 `value`.
fn time_of(value: &Value) -> time::OffsetDateTime {
    let text = value.as_str().unwrap();
    time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339).unwrap()
}

/// The body of a token named `name` granting `scopes` for `days`, without a lifetime when null.
fn token(name: &str, scopes: &[&str], days: Value) -> Value {
    let mut body = json!({ "name": name, "scopes": scopes, "expiresInDays": days });
    if days.is_null() {
        body.as_object_mut().unwrap().remove("expiresInDays");
    }
    body
}

#[tokio::test]
async fn a_token_needs_a_name_a_known_scope_and_an_allowed_lifetime() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let post = |body: Value| with_json(api(&ada, Method::POST, TOKENS), &body);
    let long = "n".repeat(61);
    let refusals = [
        (token(" ", &["read"], json!(90)), 422, "token.name"),
        (token(&long, &["read"], json!(90)), 422, "token.name"),
        (token("CI", &["read"], json!(45)), 422, "token.expiry"),
        (token("CI", &[], json!(90)), 400, "request.malformed"),
        (token("CI", &["admin"], json!(90)), 400, "request.malformed"),
        (
            token("CI", &["read"], Value::Null),
            400,
            "request.malformed",
        ),
    ];
    let mut params = Vec::new();
    for (body, status, code) in refusals {
        params.push(stack.refused(post(body), (status, code)).await);
    }
    assert_eq!(params[0], json!({ "max": 60 }));
    assert_eq!(params[2], json!({ "allowed": [30, 90, 365] }));
    for days in [30, 365] {
        create_token(&stack, &ada, ("CI", &["export"], days)).await;
    }
    assert_eq!(stack.count(TOKEN_COUNT, "ada@example.org").await, 2);
}

#[tokio::test]
async fn an_account_has_at_most_twenty_active_tokens() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    let mut first = None;
    for n in 0..20 {
        let created = create_token(&stack, &ada, (&format!("Token {n}"), &["read"], 30)).await;
        first.get_or_insert(created["id"].as_str().unwrap().to_owned());
    }
    let body = json!({ "name": "One more", "scopes": ["read"], "expiresInDays": 30 });
    let one_more = || with_json(api(&ada, Method::POST, TOKENS), &body);

    let params = stack.refused(one_more(), (409, "token.limit")).await;

    assert_eq!(params, json!({ "max": 20 }));
    create_token(&stack, &grace, ("Grace's", &["read"], 30)).await;
    let first = first.unwrap();
    stack
        .expect(204, delete(&ada, &format!("{TOKENS}/{first}")))
        .await;
    stack.created(one_more()).await;
}

#[tokio::test]
async fn the_routes_need_a_session_and_changes_the_csrf_token() {
    let stack = ApiStack::new().await;
    let body = json!({ "name": "CI", "scopes": ["read"], "expiresInDays": 30 });
    let anonymous = with_json(from_app(Method::POST, "/api/v1/tokens"), &body);
    stack
        .refused(anonymous, (401, "auth.unauthenticated"))
        .await;
    let ada = stack.sign_up("ada@example.org").await;
    let unsigned = with_json(ada.cookie(from_app(Method::POST, "/api/v1/tokens")), &body);
    stack.refused(unsigned, (403, "auth.csrf")).await;
}
