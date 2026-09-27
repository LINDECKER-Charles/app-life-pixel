use super::*;
use crate::accounts::{AccountsError, PasswordChange};

/// A change from `current` to `new`.
fn change(current: &str, new: &str) -> PasswordChange {
    PasswordChange {
        current_password: current.to_owned(),
        new_password: new.to_owned(),
    }
}

#[tokio::test]
async fn a_change_keeps_this_session_and_ends_the_others() {
    let harness = Harness::new();
    let other = harness.sign_up().await;
    let this = harness.sign_in(EMAIL, PASSWORD).await.unwrap();
    let session = harness.accounts.authenticate(&this.token).await.unwrap();
    let changed = harness
        .accounts
        .change_password(&session, change(PASSWORD, OTHER_PASSWORD));
    changed.await.unwrap();
    assert!(harness.accounts.authenticate(&this.token).await.is_ok());
    let ended = harness.accounts.authenticate(&other.token).await;
    assert_eq!(ended.unwrap_err(), AccountsError::Unauthenticated);
    assert!(harness.sign_in(EMAIL, OTHER_PASSWORD).await.is_ok());
    let emails = harness.emails(2).await;
    assert!(matches!(emails[1].message, Message::PasswordChanged));
}

#[tokio::test]
async fn a_change_needs_the_current_password_and_a_new_one_in_bounds() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let session = harness
        .accounts
        .authenticate(&signed_in.token)
        .await
        .unwrap();
    let wrong = harness
        .accounts
        .change_password(&session, change(OTHER_PASSWORD, PASSWORD));
    assert_eq!(wrong.await.unwrap_err(), AccountsError::CurrentPassword);
    let short = harness
        .accounts
        .change_password(&session, change("short", PASSWORD));
    assert_eq!(short.await.unwrap_err(), AccountsError::CurrentPassword);
    let too_short = harness
        .accounts
        .change_password(&session, change(PASSWORD, "short"));
    assert_eq!(too_short.await.unwrap_err(), AccountsError::PasswordLength);
    assert!(harness.sign_in(EMAIL, PASSWORD).await.is_ok());
}

#[tokio::test]
async fn a_password_confirms_a_sensitive_action() {
    let harness = Harness::new();
    let id = harness.sign_up().await.account.id;
    assert!(
        harness
            .accounts
            .confirm_password(id, PASSWORD.to_owned())
            .await
            .is_ok()
    );
    let wrong = harness
        .accounts
        .confirm_password(id, OTHER_PASSWORD.to_owned())
        .await;
    assert_eq!(wrong.unwrap_err(), AccountsError::CurrentPassword);
}

#[tokio::test]
async fn an_account_chooses_a_known_language_and_can_be_deleted() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let id = signed_in.account.id;
    harness.accounts.change_language(id, "fr").await.unwrap();
    assert_eq!(harness.accounts.account(id).await.unwrap().language, "fr");
    let unknown = harness
        .accounts
        .change_language(id, "tlh")
        .await
        .unwrap_err();
    assert!(matches!(unknown, AccountsError::Language { .. }));
    harness.accounts.delete_account(id).await.unwrap();
    let gone = harness.accounts.account(id).await;
    assert_eq!(gone.unwrap_err(), AccountsError::Unauthenticated);
    let session = harness.accounts.authenticate(&signed_in.token).await;
    assert_eq!(session.unwrap_err(), AccountsError::Unauthenticated);
}
