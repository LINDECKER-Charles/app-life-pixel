use super::*;
use crate::accounts::AccountsError;
use crate::ids::AccountId;

#[tokio::test]
async fn signing_up_signs_in_and_sends_the_verification_email() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let account = &signed_in.account;
    assert_eq!(account.email, EMAIL);
    assert!(!account.is_email_verified);
    assert_eq!(account.language, "en");
    assert_eq!(account.plan, "free");
    assert_eq!(account.storage.used_bytes, 0);
    assert_eq!(account.storage.limit_bytes, Some(FREE_STORAGE_BYTES));
    assert_eq!(account.created_at, START);
    let session = harness
        .accounts
        .authenticate(&signed_in.token)
        .await
        .unwrap();
    assert_eq!(session.account, *account);
    let emails = harness.emails(1).await;
    assert_eq!(emails[0].to, EMAIL);
    assert_eq!(emails[0].language, "en");
    assert!(matches!(emails[0].message, Message::VerifyEmail { .. }));
    assert_eq!(harness.event_names(), ["signed_up"]);
}

#[tokio::test]
async fn the_password_is_stored_as_an_argon2id_hash() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let record = harness.store.record(signed_in.account.id).unwrap();
    let phc = record.password_hash.as_phc();
    assert!(phc.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"), "{phc}");
    assert!(!phc.contains(PASSWORD));
}

#[tokio::test]
async fn an_address_is_trimmed_and_taken_whatever_its_case() {
    let harness = Harness::new();
    let signed_in = harness
        .accounts
        .sign_up(sign_up("  Ada@Example.com ", PASSWORD))
        .await;
    assert_eq!(signed_in.unwrap().account.email, "Ada@Example.com");
    let again = harness
        .accounts
        .sign_up(sign_up(EMAIL, OTHER_PASSWORD))
        .await;
    assert_eq!(again.unwrap_err(), AccountsError::EmailTaken);
}

#[tokio::test]
async fn a_sign_up_is_checked_address_first_then_password_then_language() {
    let harness = Harness::new();
    let both_wrong = harness.accounts.sign_up(sign_up("ada", "short")).await;
    assert_eq!(both_wrong.unwrap_err(), AccountsError::EmailInvalid);
    let short = harness.accounts.sign_up(sign_up(EMAIL, "short")).await;
    assert_eq!(short.unwrap_err(), AccountsError::PasswordLength);
    let mut klingon = sign_up(EMAIL, PASSWORD);
    klingon.language = "tlh".to_owned();
    let unknown = harness.accounts.sign_up(klingon).await.unwrap_err();
    let available = vec!["en".to_owned(), "fr".to_owned()];
    assert_eq!(unknown, AccountsError::Language { available });
    let first = AccountId::from_uuid(SequentialIds::id(1));
    assert!(harness.store.record(first).is_none());
    assert!(harness.mailer.sent().is_empty());
}

#[tokio::test]
async fn a_french_account_is_emailed_in_french() {
    let harness = Harness::new();
    let mut french = sign_up(EMAIL, PASSWORD);
    french.language = "fr".to_owned();
    harness.accounts.sign_up(french).await.unwrap();
    assert_eq!(harness.emails(1).await[0].language, "fr");
}
