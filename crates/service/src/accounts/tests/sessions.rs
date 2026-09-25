use time::Duration;

use super::*;
use crate::accounts::{AccountsError, SESSION_LIFETIME, SecretToken};

#[tokio::test]
async fn a_session_seen_within_a_day_is_not_due_for_extension() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.clock.advance(Duration::hours(23));
    let session = harness
        .accounts
        .authenticate(&signed_in.token)
        .await
        .unwrap();
    assert_eq!(session.token_hash, signed_in.token.hash());
    assert!(!session.is_due_for_extension);
}

#[tokio::test]
async fn a_session_seen_more_than_a_day_ago_slides_thirty_days_further() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.clock.advance(Duration::days(29));
    let session = harness
        .accounts
        .authenticate(&signed_in.token)
        .await
        .unwrap();
    assert!(session.is_due_for_extension);
    harness
        .accounts
        .extend_session(&session.token_hash)
        .await
        .unwrap();
    let stored = &harness.sessions.sessions_of(signed_in.account.id)[0];
    assert_eq!(stored.last_seen_at, START + Duration::days(29));
    assert_eq!(
        stored.expires_at,
        START + Duration::days(29) + SESSION_LIFETIME
    );
    harness.clock.advance(Duration::days(20));
    let later = harness
        .accounts
        .authenticate(&signed_in.token)
        .await
        .unwrap();
    assert!(later.is_due_for_extension);
}

#[tokio::test]
async fn a_session_unseen_for_thirty_days_has_ended() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.clock.advance(SESSION_LIFETIME);
    let ended = harness.accounts.authenticate(&signed_in.token).await;
    assert_eq!(ended.unwrap_err(), AccountsError::Unauthenticated);
}

#[tokio::test]
async fn an_unknown_token_is_not_a_session() {
    let harness = Harness::new();
    harness.sign_up().await;
    let unknown = harness
        .accounts
        .authenticate(&SecretToken::generate())
        .await;
    assert_eq!(unknown.unwrap_err(), AccountsError::Unauthenticated);
}

#[tokio::test]
async fn signing_out_ends_the_session_alone() {
    let harness = Harness::new();
    let first = harness.sign_up().await;
    let second = harness.sign_in(EMAIL, PASSWORD).await.unwrap();
    harness.accounts.sign_out(&first.token).await.unwrap();
    let ended = harness.accounts.authenticate(&first.token).await;
    assert_eq!(ended.unwrap_err(), AccountsError::Unauthenticated);
    assert!(harness.accounts.authenticate(&second.token).await.is_ok());
}

#[tokio::test]
async fn the_sessions_of_a_suspended_account_stop_working() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.store.suspend(signed_in.account.id);
    let suspended = harness.accounts.authenticate(&signed_in.token).await;
    assert_eq!(suspended.unwrap_err(), AccountsError::AccountSuspended);
    let account = harness.accounts.account(signed_in.account.id).await;
    assert_eq!(account.unwrap_err(), AccountsError::AccountSuspended);
}

#[tokio::test]
async fn the_purge_deletes_expired_sessions_and_tokens_only() {
    let harness = Harness::new();
    let old = harness.sign_up().await;
    harness.clock.advance(Duration::days(8));
    let recent = harness.sign_in(EMAIL, PASSWORD).await.unwrap();
    assert_eq!(harness.accounts.purge_expired().await.unwrap(), (0, 1));
    harness.clock.advance(Duration::days(23));
    assert_eq!(harness.accounts.purge_expired().await.unwrap(), (1, 0));
    assert!(harness.tokens.is_empty());
    let sessions = harness.sessions.sessions_of(old.account.id);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].token_hash, recent.token.hash());
}
