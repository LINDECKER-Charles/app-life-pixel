use time::Duration;

use super::*;
use crate::accounts::{
    AccountsError, PasswordReset, RESET_PASSWORD_LIFETIME, RESET_PASSWORD_PATH,
    VERIFY_EMAIL_LIFETIME, VERIFY_EMAIL_PATH,
};

/// How long a test waits before deciding no email left.
const NO_MAIL_WAIT: std::time::Duration = std::time::Duration::from_millis(200);

/// Asks for a reset link for [`EMAIL`] and returns its token.
async fn reset_token(harness: &Harness, emails_before: usize) -> String {
    harness.accounts.request_password_reset(EMAIL);
    let emails = harness.emails(emails_before + 1).await;
    let message = &emails[emails_before].message;
    assert!(matches!(message, Message::ResetPassword { .. }));
    token_in(message, RESET_PASSWORD_PATH)
}

/// A reset with `token` to `password`.
fn reset(token: &str, password: &str) -> PasswordReset {
    PasswordReset {
        token: token.to_owned(),
        password: password.to_owned(),
    }
}

#[tokio::test]
async fn the_emailed_link_verifies_the_address_once() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let token = token_in(&harness.emails(1).await[0].message, VERIFY_EMAIL_PATH);
    harness.accounts.verify_email(&token).await.unwrap();
    let account = harness
        .accounts
        .account(signed_in.account.id)
        .await
        .unwrap();
    assert!(account.is_email_verified);
    let again = harness.accounts.verify_email(&token).await;
    assert_eq!(again.unwrap_err(), AccountsError::TokenInvalid);
}

#[tokio::test]
async fn a_verification_link_expires_after_seven_days() {
    let harness = Harness::new();
    harness.sign_up().await;
    let token = token_in(&harness.emails(1).await[0].message, VERIFY_EMAIL_PATH);
    harness.clock.advance(VERIFY_EMAIL_LIFETIME);
    let expired = harness.accounts.verify_email(&token).await;
    assert_eq!(expired.unwrap_err(), AccountsError::TokenInvalid);
}

#[tokio::test]
async fn a_token_works_for_its_own_purpose_only() {
    let harness = Harness::new();
    harness.sign_up().await;
    let verification = token_in(&harness.emails(1).await[0].message, VERIFY_EMAIL_PATH);
    let as_reset = harness
        .accounts
        .confirm_password_reset(reset(&verification, OTHER_PASSWORD));
    assert_eq!(as_reset.await.unwrap_err(), AccountsError::TokenInvalid);
    for malformed in ["", "short", &"x".repeat(44)] {
        let refused = harness.accounts.verify_email(malformed).await;
        assert_eq!(refused.unwrap_err(), AccountsError::TokenInvalid);
    }
    assert!(harness.accounts.verify_email(&verification).await.is_ok());
}

#[tokio::test]
async fn the_verification_email_is_sent_again_until_the_address_is_verified() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let first = token_in(&harness.emails(1).await[0].message, VERIFY_EMAIL_PATH);
    harness
        .accounts
        .resend_verification(signed_in.account.id)
        .await
        .unwrap();
    let second = token_in(&harness.emails(2).await[1].message, VERIFY_EMAIL_PATH);
    assert_ne!(first, second);
    harness.accounts.verify_email(&second).await.unwrap();
    harness
        .accounts
        .resend_verification(signed_in.account.id)
        .await
        .unwrap();
    assert_eq!(harness.mailer.wait_for(3, NO_MAIL_WAIT).await.len(), 2);
}

#[tokio::test]
async fn a_reset_sets_the_password_and_ends_every_session() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let token = reset_token(&harness, 1).await;
    let confirmed = harness
        .accounts
        .confirm_password_reset(reset(&token, OTHER_PASSWORD));
    confirmed.await.unwrap();
    let ended = harness.accounts.authenticate(&signed_in.token).await;
    assert_eq!(ended.unwrap_err(), AccountsError::Unauthenticated);
    assert!(harness.sign_in(EMAIL, OTHER_PASSWORD).await.is_ok());
    let old = harness.sign_in(EMAIL, PASSWORD).await;
    assert_eq!(old.unwrap_err(), AccountsError::InvalidCredentials);
    assert!(matches!(
        harness.emails(3).await[2].message,
        Message::PasswordChanged
    ));
    let again = harness
        .accounts
        .confirm_password_reset(reset(&token, PASSWORD))
        .await;
    assert_eq!(again.unwrap_err(), AccountsError::TokenInvalid);
}

#[tokio::test]
async fn a_reset_link_expires_after_an_hour() {
    let harness = Harness::new();
    harness.sign_up().await;
    let token = reset_token(&harness, 1).await;
    harness
        .clock
        .advance(RESET_PASSWORD_LIFETIME + Duration::seconds(1));
    let expired = harness
        .accounts
        .confirm_password_reset(reset(&token, OTHER_PASSWORD));
    assert_eq!(expired.await.unwrap_err(), AccountsError::TokenInvalid);
}

#[tokio::test]
async fn a_new_password_out_of_bounds_keeps_the_token() {
    let harness = Harness::new();
    harness.sign_up().await;
    let token = reset_token(&harness, 1).await;
    let short = harness
        .accounts
        .confirm_password_reset(reset(&token, "short"))
        .await;
    assert_eq!(short.unwrap_err(), AccountsError::PasswordLength);
    let confirmed = harness
        .accounts
        .confirm_password_reset(reset(&token, OTHER_PASSWORD));
    assert!(confirmed.await.is_ok());
}

#[tokio::test]
async fn no_reset_link_leaves_for_an_unknown_or_suspended_address() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.emails(1).await;
    harness.accounts.request_password_reset("grace@example.com");
    harness.accounts.request_password_reset("not an address");
    harness.store.suspend(signed_in.account.id);
    harness.accounts.request_password_reset(EMAIL);
    assert_eq!(harness.mailer.wait_for(2, NO_MAIL_WAIT).await.len(), 1);
}
