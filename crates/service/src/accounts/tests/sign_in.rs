use super::*;
use crate::accounts::AccountsError;

#[tokio::test]
async fn the_right_password_opens_a_new_session() {
    let harness = Harness::new();
    let first = harness.sign_up().await;
    let second = harness.sign_in("ADA@example.com", PASSWORD).await.unwrap();
    assert_ne!(second.token, first.token);
    assert_eq!(second.account, first.account);
    assert_eq!(harness.sessions.sessions_of(first.account.id).len(), 2);
    assert_eq!(harness.event_names(), ["signed_up", "signed_in"]);
}

#[tokio::test]
async fn a_wrong_password_and_an_unknown_address_are_told_alike() {
    let harness = Harness::new();
    harness.sign_up().await;
    let failures = [
        harness.sign_in(EMAIL, OTHER_PASSWORD).await,
        harness.sign_in("grace@example.com", PASSWORD).await,
        harness.sign_in("not an address", PASSWORD).await,
        harness.sign_in(EMAIL, "short").await,
    ];
    for failure in failures {
        assert_eq!(failure.unwrap_err(), AccountsError::InvalidCredentials);
    }
    assert_eq!(harness.event_names(), ["signed_up"]);
}

#[tokio::test]
async fn a_suspended_account_is_told_so_once_the_password_is_right() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    harness.store.suspend(signed_in.account.id);
    let wrong = harness.sign_in(EMAIL, OTHER_PASSWORD).await;
    assert_eq!(wrong.unwrap_err(), AccountsError::InvalidCredentials);
    let right = harness.sign_in(EMAIL, PASSWORD).await;
    assert_eq!(right.unwrap_err(), AccountsError::AccountSuspended);
    assert_eq!(harness.sessions.sessions_of(signed_in.account.id).len(), 1);
}

#[tokio::test]
async fn a_hash_made_with_older_parameters_is_made_again() {
    let cheap = PasswordHashing::with_costs(8_192, 1, 1).unwrap();
    let harness = Harness::hashing(cheap);
    let signed_in = harness.sign_up().await;
    let old = harness
        .store
        .record(signed_in.account.id)
        .unwrap()
        .password_hash;
    assert!(old.as_phc().starts_with("$argon2id$v=19$m=8192,t=1,p=1$"));
    let today = settings(PasswordHashing::standard());
    let upgraded = Accounts::new(harness.accounts.ports.clone(), today);
    let credentials = Credentials {
        email: EMAIL.to_owned(),
        password: PASSWORD.to_owned(),
    };
    upgraded.sign_in(credentials).await.unwrap();
    let new = harness
        .store
        .record(signed_in.account.id)
        .unwrap()
        .password_hash;
    assert!(new.as_phc().starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    assert!(harness.sign_in(EMAIL, PASSWORD).await.is_ok());
}

#[tokio::test]
async fn a_hash_with_today_parameters_is_kept() {
    let harness = Harness::new();
    let signed_in = harness.sign_up().await;
    let before = harness
        .store
        .record(signed_in.account.id)
        .unwrap()
        .password_hash;
    harness.sign_in(EMAIL, PASSWORD).await.unwrap();
    let after = harness
        .store
        .record(signed_in.account.id)
        .unwrap()
        .password_hash;
    assert_eq!(after.as_phc(), before.as_phc());
}
