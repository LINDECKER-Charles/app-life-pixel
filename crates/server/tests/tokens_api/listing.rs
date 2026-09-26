//! `GET /tokens` and `DELETE /tokens/{id}`: the account's active tokens, never their secret,
//! and revocation.

use serde_json::Value;

use crate::TOKENS;
use crate::stack::{ApiStack, delete, get};
use crate::tokens::create_token;

/// An id no token has.
const UNKNOWN: &str = "0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e";

/// The names of the account's listed tokens.
fn names(list: &Value) -> Vec<&str> {
    let tokens = list.as_array().unwrap();
    tokens
        .iter()
        .map(|token| token["name"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn the_list_shows_the_active_tokens_newest_first_without_their_secret() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    create_token(&stack, &ada, ("Laptop", &["read"], 30)).await;
    let created = create_token(&stack, &ada, ("CI", &["read", "export"], 365)).await;
    create_token(&stack, &grace, ("Grace's", &["read"], 30)).await;

    let list = stack.expect(200, get(&ada, TOKENS)).await.json();

    assert_eq!(names(&list), ["CI", "Laptop"]);
    let listed = &list[0];
    for field in ["id", "name", "prefix", "scopes", "createdAt", "expiresAt"] {
        assert_eq!(listed[field], created[field], "{field}");
    }
    assert_eq!(listed["lastUsedAt"], Value::Null);
    let secret = created["token"].as_str().unwrap();
    assert!(!list.to_string().contains(&secret[11..]));
    assert!(listed.get("token").is_none());
}

#[tokio::test]
async fn a_revoked_token_leaves_the_list_and_cannot_be_revoked_again() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let grace = stack.sign_up("grace@example.org").await;
    let created = create_token(&stack, &ada, ("Laptop", &["read"], 30)).await;
    let path = format!("{TOKENS}/{}", created["id"].as_str().unwrap());

    stack
        .refused(delete(&grace, &path), (404, "token.not_found"))
        .await;
    stack.expect(204, delete(&ada, &path)).await;

    let list = stack.expect(200, get(&ada, TOKENS)).await.json();
    assert_eq!(names(&list), Vec::<&str>::new());
    stack
        .refused(delete(&ada, &path), (404, "token.not_found"))
        .await;
    let unknown = delete(&ada, &format!("{TOKENS}/{UNKNOWN}"));
    stack.refused(unknown, (404, "token.not_found")).await;
    let malformed = delete(&ada, &format!("{TOKENS}/not-an-id"));
    stack.refused(malformed, (400, "request.malformed")).await;
}
