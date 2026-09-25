//! Signing in with an address and a password, in the same time whether the address has an
//! account or not.

use crate::accounts::hashing::Verification;
use crate::accounts::ports::{AccountRecord, AccountStatus};
use crate::accounts::values::{EmailAddress, Password};
use crate::accounts::{Accounts, AccountsError, SignedIn};

/// A session was opened with a password.
const SIGNED_IN: &str = "signed_in";

/// What a person signs in with.
#[derive(Clone, Debug)]
pub struct Credentials {
    /// The address.
    pub email: String,
    /// The password.
    pub password: String,
}

impl Accounts {
    /// Opens a session when the password is the account's. An unknown address still spends one
    /// hash, so that timing says nothing of it; a hash made with older parameters is made again.
    ///
    /// # Errors
    ///
    /// `auth.invalid_credentials` for an unknown address or a wrong password alike,
    /// `auth.account_suspended` once the password is right, `service.unavailable`.
    pub async fn sign_in(&self, credentials: Credentials) -> Result<SignedIn, AccountsError> {
        let email = EmailAddress::parse(&credentials.email);
        let password = Password::parse(credentials.password);
        let (Ok(email), Ok(password)) = (email, password) else {
            return Err(AccountsError::InvalidCredentials);
        };
        let Some(account) = self.ports.accounts.find_by_email(&email).await? else {
            self.hashing().spend(&password).await;
            return Err(AccountsError::InvalidCredentials);
        };
        let verification = self
            .hashing()
            .verify(&password, &account.password_hash)
            .await?;
        let Verification::Match { needs_rehash } = verification else {
            return Err(AccountsError::InvalidCredentials);
        };
        if account.status == AccountStatus::Suspended {
            return Err(AccountsError::AccountSuspended);
        }
        if needs_rehash {
            self.rehash(&account, &password).await;
        }
        let signed_in = self.open_session(&account).await?;
        self.record(SIGNED_IN, &account);
        Ok(signed_in)
    }

    /// Stores a hash of `password` with today's parameters; a failure is logged, and the old
    /// hash kept.
    async fn rehash(&self, account: &AccountRecord, password: &Password) {
        let Ok(hash) = self.hashing().hash(password).await else {
            return;
        };
        let accounts = &self.ports.accounts;
        let stored = accounts.set_password_hash(account.id, &hash).await;
        stored.unwrap_or_else(|error| tracing::warn!(%error, "a password hash was not upgraded"));
    }
}
