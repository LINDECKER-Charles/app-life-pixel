//! The account itself, for the routes that build on a session: reading it, choosing its
//! language, deleting it.

use crate::accounts::ports::AccountStatus;
use crate::accounts::values::Language;
use crate::accounts::{Account, Accounts, AccountsError};
use crate::ids::AccountId;

impl Accounts {
    /// The account `id`, while it is active: what a session or an access token acts for.
    ///
    /// # Errors
    ///
    /// `auth.unauthenticated` when it is gone, `auth.account_suspended`, `service.unavailable`.
    pub async fn account(&self, id: AccountId) -> Result<Account, AccountsError> {
        let record = self.ports.accounts.get(id).await?;
        let record = record.ok_or(AccountsError::Unauthenticated)?;
        if record.status == AccountStatus::Suspended {
            return Err(AccountsError::AccountSuspended);
        }
        Ok(self.view(&record))
    }

    /// Sets the language of the account `id` to `code`.
    ///
    /// # Errors
    ///
    /// `account.language`, `service.unavailable`.
    pub async fn change_language(&self, id: AccountId, code: &str) -> Result<(), AccountsError> {
        let language = Language::parse(code, &self.settings.languages)?;
        Ok(self.ports.accounts.set_language(id, &language).await?)
    }

    /// Deletes the account `id`, and with it its sessions and tokens; the caller deletes its
    /// library first.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn delete_account(&self, id: AccountId) -> Result<(), AccountsError> {
        Ok(self.ports.accounts.delete(id).await?)
    }
}
