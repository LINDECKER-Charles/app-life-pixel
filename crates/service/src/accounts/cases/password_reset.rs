//! Resetting a forgotten password: a link by email, then a new password that ends every session.

use crate::accounts::ports::{AccountStatus, Message, TokenPurpose};
use crate::accounts::values::{EmailAddress, Password};
use crate::accounts::{Accounts, AccountsError, RESET_PASSWORD_PATH};

/// What confirms a reset: the emailed token and the new password.
#[derive(Clone, Debug)]
pub struct PasswordReset {
    /// The token of the emailed link.
    pub token: String,
    /// The new password.
    pub password: String,
}

impl Accounts {
    /// Sends a link setting a new password to `email`, when an active account has it. The work
    /// runs on a task of its own, so that the caller learns nothing of the address — neither
    /// from the answer nor from its timing.
    pub fn request_password_reset(&self, email: &str) {
        let Ok(email) = EmailAddress::parse(email) else {
            return;
        };
        let accounts = self.clone();
        tokio::spawn(async move {
            if let Err(error) = accounts.send_reset_link(&email).await {
                tracing::warn!(%error, "a password reset link was not sent");
            }
        });
    }

    /// Issues a reset token for the active account of `email`, if any, and sends its link.
    pub(crate) async fn send_reset_link(&self, email: &EmailAddress) -> Result<(), AccountsError> {
        let account = self.ports.accounts.find_by_email(email).await?;
        let Some(account) = account.filter(|account| account.status == AccountStatus::Active)
        else {
            return Ok(());
        };
        let token = self
            .issue_token(&account, TokenPurpose::ResetPassword)
            .await?;
        let link = self.link(RESET_PASSWORD_PATH, &token);
        self.dispatch(&account, Message::ResetPassword { link });
        Ok(())
    }

    /// Sets the new password of the account an unused, unexpired reset token acts for, spends
    /// the token, ends every session of the account, and tells its owner by email.
    ///
    /// # Errors
    ///
    /// `auth.password_length`, then `auth.token_invalid`, `service.unavailable`.
    pub async fn confirm_password_reset(&self, reset: PasswordReset) -> Result<(), AccountsError> {
        let password = Password::parse(reset.password)?;
        let account = self
            .spend_token(&reset.token, TokenPurpose::ResetPassword)
            .await?;
        let hash = self.hashing().hash(&password).await?;
        self.ports
            .accounts
            .set_password_hash(account.id, &hash)
            .await?;
        self.ports.sessions.delete_all(account.id).await?;
        self.dispatch(&account, Message::PasswordChanged);
        Ok(())
    }
}
