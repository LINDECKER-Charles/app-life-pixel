//! Changing a known password, and confirming one before a sensitive action.

use crate::accounts::hashing::Verification;
use crate::accounts::ports::{AccountRecord, Message};
use crate::accounts::values::Password;
use crate::accounts::{Accounts, AccountsError, Authenticated};
use crate::ids::AccountId;

/// A change of password: the current one, then the new one.
#[derive(Clone, Debug)]
pub struct PasswordChange {
    /// The current password.
    pub current_password: String,
    /// The new password.
    pub new_password: String,
}

impl Accounts {
    /// Replaces the password of the signed-in account, ends its other sessions — `session`
    /// stays open —, and tells its owner by email.
    ///
    /// # Errors
    ///
    /// `auth.password_length` for the new password, `auth.current_password`,
    /// `auth.unauthenticated` when the account is gone, `service.unavailable`.
    pub async fn change_password(
        &self,
        session: &Authenticated,
        change: PasswordChange,
    ) -> Result<(), AccountsError> {
        let new_password = Password::parse(change.new_password)?;
        let account = self.current_password(session.account.id, change.current_password);
        let account = account.await?;
        let hash = self.hashing().hash(&new_password).await?;
        self.ports
            .accounts
            .set_password_hash(account.id, &hash)
            .await?;
        let others = self
            .ports
            .sessions
            .delete_others(account.id, &session.token_hash);
        others.await?;
        self.dispatch(&account, Message::PasswordChanged);
        Ok(())
    }

    /// Checks that `password` is the account's, as an action that destroys asks — deleting the
    /// account, for one.
    ///
    /// # Errors
    ///
    /// `auth.current_password`, `auth.unauthenticated` when the account is gone,
    /// `service.unavailable`.
    pub async fn confirm_password(
        &self,
        account: AccountId,
        password: String,
    ) -> Result<(), AccountsError> {
        self.current_password(account, password).await.map(|_| ())
    }

    /// The account, when `password` is its password.
    async fn current_password(
        &self,
        account: AccountId,
        password: String,
    ) -> Result<AccountRecord, AccountsError> {
        let record = self.ports.accounts.get(account).await?;
        let record = record.ok_or(AccountsError::Unauthenticated)?;
        let password = Password::parse(password).map_err(|_| AccountsError::CurrentPassword)?;
        match self
            .hashing()
            .verify(&password, &record.password_hash)
            .await?
        {
            Verification::Match { .. } => Ok(record),
            Verification::Mismatch => Err(AccountsError::CurrentPassword),
        }
    }
}
