//! Accounts in memory.

use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::accounts::hashing::PasswordHash;
use crate::accounts::ports::{
    AccountRecord, AccountStatus, AccountStore, AccountStoreError, NewAccount,
};
use crate::accounts::values::{EmailAddress, Language};
use crate::ids::AccountId;

/// The plan of a new account.
const FREE_PLAN: &str = "free";

/// Accounts in memory; addresses compared whatever their case. Deleting an account leaves its
/// sessions and tokens in their own stores, where Postgres would cascade.
#[derive(Debug, Default)]
pub struct InMemoryAccountStore {
    accounts: Mutex<Vec<AccountRecord>>,
}

impl InMemoryAccountStore {
    /// A store with no account.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The account `id` as stored, for tests to inspect.
    #[must_use]
    pub fn record(&self, id: AccountId) -> Option<AccountRecord> {
        self.lock().iter().find(|account| account.id == id).cloned()
    }

    /// Suspends the account `id`, as an administrator would.
    pub fn suspend(&self, id: AccountId) {
        self.update(id, |account| account.status = AccountStatus::Suspended);
    }

    fn lock(&self) -> MutexGuard<'_, Vec<AccountRecord>> {
        self.accounts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn update(&self, id: AccountId, change: impl FnOnce(&mut AccountRecord)) {
        if let Some(account) = self.lock().iter_mut().find(|account| account.id == id) {
            change(account);
        }
    }
}

#[async_trait]
impl AccountStore for InMemoryAccountStore {
    async fn create(&self, account: NewAccount) -> Result<AccountRecord, AccountStoreError> {
        let mut accounts = self.lock();
        let address = account.email.as_str().to_lowercase();
        if accounts
            .iter()
            .any(|known| known.email.to_lowercase() == address)
        {
            return Err(AccountStoreError::EmailTaken);
        }
        let record = AccountRecord {
            id: account.id,
            email: account.email.as_str().to_owned(),
            password_hash: account.password_hash,
            email_verified_at: None,
            language: account.language.as_str().to_owned(),
            status: AccountStatus::Active,
            plan: FREE_PLAN.to_owned(),
            storage_used_bytes: 0,
            created_at: account.created_at,
        };
        accounts.push(record.clone());
        Ok(record)
    }

    async fn get(&self, id: AccountId) -> Result<Option<AccountRecord>, AccountStoreError> {
        Ok(self.record(id))
    }

    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<AccountRecord>, AccountStoreError> {
        let address = email.as_str().to_lowercase();
        let accounts = self.lock();
        let found = accounts
            .iter()
            .find(|known| known.email.to_lowercase() == address);
        Ok(found.cloned())
    }

    async fn set_password_hash(
        &self,
        id: AccountId,
        hash: &PasswordHash,
    ) -> Result<(), AccountStoreError> {
        self.update(id, |account| account.password_hash = hash.clone());
        Ok(())
    }

    async fn mark_email_verified(
        &self,
        id: AccountId,
        at: OffsetDateTime,
    ) -> Result<(), AccountStoreError> {
        self.update(id, |account| {
            account.email_verified_at.get_or_insert(at);
        });
        Ok(())
    }

    async fn set_language(
        &self,
        id: AccountId,
        language: &Language,
    ) -> Result<(), AccountStoreError> {
        self.update(id, |account| {
            language.as_str().clone_into(&mut account.language)
        });
        Ok(())
    }

    async fn delete(&self, id: AccountId) -> Result<(), AccountStoreError> {
        self.lock().retain(|account| account.id != id);
        Ok(())
    }
}
