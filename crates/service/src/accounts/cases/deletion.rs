//! Deleting an account at its owner's request: its password again, then its documents, then the
//! account itself — its sessions, tokens and library rows go with it.

use crate::accounts::{Accounts, AccountsError};
use crate::ids::AccountId;
use crate::library::Library;
use crate::owner::Owner;
use crate::ports::ProductEvent;

/// An account was deleted by its owner.
const ACCOUNT_DELETED: &str = "account_deleted";

/// A deletion asked by the owner of an account, confirmed with its password.
#[derive(Clone)]
pub struct AccountDeletion {
    /// The account to delete.
    pub account: AccountId,
    /// Its password, asked again.
    pub password: String,
}

impl Accounts {
    /// Deletes the account once its password is confirmed: first every document of `library`,
    /// then the account, whose sessions, tokens and remaining rows go with it. Records
    /// `account_deleted`.
    ///
    /// # Errors
    ///
    /// `auth.current_password`, `auth.unauthenticated` when the account is gone,
    /// `service.unavailable`.
    pub async fn delete_with_library(
        &self,
        library: &Library,
        deletion: AccountDeletion,
    ) -> Result<(), AccountsError> {
        let account = deletion.account;
        self.confirm_password(account, deletion.password).await?;
        let emptied = library.delete_everything(&Owner::Account(account)).await;
        emptied.map_err(|_| AccountsError::Unavailable)?;
        self.delete_account(account).await?;
        self.ports.events.record(ProductEvent {
            name: ACCOUNT_DELETED,
            account: Some(account),
            properties: Vec::new(),
        });
        Ok(())
    }
}
