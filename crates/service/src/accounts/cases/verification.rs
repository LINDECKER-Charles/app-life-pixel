//! Verifying an address: the emailed link, and sending it again. An unverified address blocks
//! nothing in V1.

use crate::accounts::ports::{AccountRecord, Message, TokenPurpose};
use crate::accounts::{Accounts, AccountsError, VERIFY_EMAIL_PATH};
use crate::ids::AccountId;

impl Accounts {
    /// Verifies the address of the account an unused, unexpired verification `token` acts for,
    /// and spends it.
    ///
    /// # Errors
    ///
    /// `auth.token_invalid`, `service.unavailable`.
    pub async fn verify_email(&self, token: &str) -> Result<(), AccountsError> {
        let account = self.spend_token(token, TokenPurpose::VerifyEmail).await?;
        let verified = self
            .ports
            .accounts
            .mark_email_verified(account.id, self.now());
        Ok(verified.await?)
    }

    /// Sends the verification email again, with a new link; nothing when the address is already
    /// verified.
    ///
    /// # Errors
    ///
    /// `auth.unauthenticated` when the account is gone, `service.unavailable`.
    pub async fn resend_verification(&self, account: AccountId) -> Result<(), AccountsError> {
        let record = self.ports.accounts.get(account).await?;
        let record = record.ok_or(AccountsError::Unauthenticated)?;
        if record.email_verified_at.is_some() {
            return Ok(());
        }
        self.send_verification(&record).await
    }

    /// Issues a verification token for `account` and sends its link.
    pub(super) async fn send_verification(
        &self,
        account: &AccountRecord,
    ) -> Result<(), AccountsError> {
        let token = self.issue_token(account, TokenPurpose::VerifyEmail).await?;
        let link = self.link(VERIFY_EMAIL_PATH, &token);
        self.dispatch(account, Message::VerifyEmail { link });
        Ok(())
    }
}
