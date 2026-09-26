//! The token use cases over the in-memory adapter.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

use std::sync::Arc;

use life_pixel_core::limits::MAX_ACTIVE_TOKENS;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::memory::InMemoryTokenStore;
use super::{NewToken, TokenScope, Tokens, TokensError, TokensPorts};
use crate::Coded;
use crate::ids::AccountId;
use crate::memory::{FixedClock, SequentialIds};

/// When the tests start.
const START: OffsetDateTime = datetime!(2026-09-01 12:00 UTC);

struct Harness {
    tokens: Tokens,
    store: Arc<InMemoryTokenStore>,
    clock: Arc<FixedClock>,
}

impl Harness {
    fn new() -> Self {
        let store = Arc::new(InMemoryTokenStore::new());
        let clock = Arc::new(FixedClock::new(START));
        let tokens = Tokens::new(TokensPorts {
            store: store.clone(),
            clock: clock.clone(),
            ids: Arc::new(SequentialIds::new()),
        });
        Self {
            tokens,
            store,
            clock,
        }
    }
}

fn account(n: u128) -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(n))
}

fn request(name: &str, scopes: &[TokenScope], expires_in_days: u16) -> NewToken {
    NewToken {
        name: name.to_owned(),
        scopes: scopes.to_vec(),
        expires_in_days,
    }
}

fn claude() -> NewToken {
    request(" Claude Code ", &[TokenScope::Write, TokenScope::Read], 90)
}

#[tokio::test]
async fn a_created_token_authenticates_its_account_with_its_scopes() {
    let harness = Harness::new();
    let created = harness.tokens.create(account(1), claude()).await.unwrap();

    assert_eq!(created.record.name, "Claude Code");
    assert_eq!(created.record.scopes, [TokenScope::Read, TokenScope::Write]);
    assert_eq!(created.record.expires_at, START + Duration::days(90));
    assert_eq!(created.record.prefix, created.secret.expose()[..11]);
    let token = harness
        .tokens
        .authenticate(created.secret.expose())
        .await
        .unwrap();
    assert_eq!(token.account, account(1));
    assert!(token.require(TokenScope::Write).is_ok());
    let refused = token.require(TokenScope::Export).unwrap_err();
    assert_eq!(refused.code(), "token.scope");
    assert_eq!(refused.params()["required"], "export");
}

#[tokio::test]
async fn names_scopes_and_lifetimes_are_checked() {
    let harness = Harness::new();
    let long = "n".repeat(61);
    let cases = [
        (request("  ", &[TokenScope::Read], 30), TokensError::Name),
        (request(&long, &[TokenScope::Read], 30), TokensError::Name),
        (request("agent", &[], 30), TokensError::NoScope),
        (
            request("agent", &[TokenScope::Read], 7),
            TokensError::Expiry,
        ),
    ];
    for (request, expected) in cases {
        let refused = harness
            .tokens
            .create(account(1), request)
            .await
            .unwrap_err();
        assert_eq!(refused, expected);
    }
    let expiry = TokensError::Expiry.params();
    assert_eq!(expiry["allowed"], serde_json::json!([30, 90, 365]));
}

#[tokio::test]
async fn an_account_holds_at_most_the_active_tokens_the_limit_allows() {
    let harness = Harness::new();
    for _ in 0..MAX_ACTIVE_TOKENS {
        harness.tokens.create(account(1), claude()).await.unwrap();
    }
    let refused = harness
        .tokens
        .create(account(1), claude())
        .await
        .unwrap_err();
    assert_eq!(refused, TokensError::Limit);
    assert!(harness.tokens.create(account(2), claude()).await.is_ok());
    let first = harness.tokens.list(account(1)).await.unwrap()[0].id;
    harness.tokens.revoke(account(1), first).await.unwrap();
    assert!(harness.tokens.create(account(1), claude()).await.is_ok());
}

#[tokio::test]
async fn a_revoked_or_expired_token_no_longer_authenticates() {
    let harness = Harness::new();
    let revoked = harness.tokens.create(account(1), claude()).await.unwrap();
    let expiring = request("short", &[TokenScope::Read], 30);
    let expiring = harness.tokens.create(account(1), expiring).await.unwrap();

    let id = revoked.record.id;
    assert_eq!(
        harness.tokens.revoke(account(2), id).await,
        Err(TokensError::NotFound)
    );
    harness.tokens.revoke(account(1), id).await.unwrap();
    assert_eq!(
        harness.tokens.revoke(account(1), id).await,
        Err(TokensError::NotFound)
    );
    let secret = revoked.secret.expose();
    assert_eq!(
        harness.tokens.authenticate(secret).await,
        Err(TokensError::Invalid)
    );
    harness.clock.advance(Duration::days(30));
    let secret = expiring.secret.expose();
    assert_eq!(
        harness.tokens.authenticate(secret).await,
        Err(TokensError::Invalid)
    );
    assert!(harness.tokens.list(account(1)).await.unwrap().is_empty());
    assert_eq!(harness.tokens.list_all(account(1)).await.unwrap().len(), 2);
}

#[tokio::test]
async fn unknown_texts_are_invalid_and_suspended_accounts_refused() {
    let harness = Harness::new();
    for text in ["", "lp_pat_short", "Bearer lp_pat_x"] {
        let refused = harness.tokens.authenticate(text).await.unwrap_err();
        assert_eq!(refused, TokensError::Invalid, "{text}");
    }
    let created = harness.tokens.create(account(1), claude()).await.unwrap();
    harness.store.suspend(account(1));
    let refused = harness.tokens.authenticate(created.secret.expose()).await;
    assert_eq!(refused, Err(TokensError::AccountSuspended));
}

#[tokio::test]
async fn the_last_use_is_recorded_at_most_once_a_minute() {
    let harness = Harness::new();
    let created = harness.tokens.create(account(1), claude()).await.unwrap();
    let secret = created.secret.expose();
    let last_use = || async { harness.tokens.list(account(1)).await.unwrap()[0].last_used_at };

    assert_eq!(last_use().await, None);
    harness.tokens.authenticate(secret).await.unwrap();
    assert_eq!(last_use().await, Some(START));
    harness.clock.advance(Duration::seconds(59));
    harness.tokens.authenticate(secret).await.unwrap();
    assert_eq!(last_use().await, Some(START));
    harness.clock.advance(Duration::seconds(1));
    harness.tokens.authenticate(secret).await.unwrap();
    assert_eq!(last_use().await, Some(START + Duration::minutes(1)));
}
