//! Signing up: a new account, signed in at once, and the email that verifies its address.

use crate::accounts::ports::{AccountRecord, NewAccount};
use crate::accounts::values::{EmailAddress, Language, Password};
use crate::accounts::{Accounts, AccountsError, SignedIn};
use crate::ids::AccountId;

/// A new account was created.
const SIGNED_UP: &str = "signed_up";

/// What a person signs up with.
#[derive(Clone, Debug)]
pub struct SignUp {
    /// The address.
    pub email: String,
    /// The password.
    pub password: String,
    /// The language code of the interface they use.
    pub language: String,
}

impl Accounts {
    /// Creates the account, opens its session, and sends the email that verifies its address.
    /// Signing up says when an address is taken: the `sign_up` rate limit bounds what that
    /// tells.
    ///
    /// # Errors
    ///
    /// `auth.email_invalid`, `auth.password_length`, `account.language`, `auth.email_taken`,
    /// `service.unavailable`.
    pub async fn sign_up(&self, request: SignUp) -> Result<SignedIn, AccountsError> {
        let email = EmailAddress::parse(&request.email)?;
        let password = Password::parse(request.password)?;
        let language = Language::parse(&request.language, &self.settings.languages)?;
        let password_hash = self.hashing().hash(&password).await?;
        let new = NewAccount {
            id: AccountId::from_uuid(self.ports.ids.new_id()),
            email,
            password_hash,
            language,
            created_at: self.now(),
        };
        let account = self.ports.accounts.create(new).await?;
        let signed_in = self.open_session(&account).await?;
        self.welcome(&account).await;
        self.record(SIGNED_UP, &account);
        Ok(signed_in)
    }

    /// Sends the email that verifies a new account's address; a failure is logged, and the
    /// account kept: it can ask for the email again.
    async fn welcome(&self, account: &AccountRecord) {
        if let Err(error) = self.send_verification(account).await {
            tracing::warn!(%error, "the verification email of a new account was not sent");
        }
    }
}
