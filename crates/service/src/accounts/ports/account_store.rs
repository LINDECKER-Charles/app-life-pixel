//! Where accounts are kept: their address, password hash, language, status, plan and usage.

use async_trait::async_trait;
use thiserror::Error;
use time::OffsetDateTime;

use crate::accounts::hashing::PasswordHash;
use crate::accounts::values::{EmailAddress, Language};
use crate::ids::AccountId;

/// Whether an account may sign in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountStatus {
    /// It may sign in, and its sessions work.
    Active,
    /// It may not, and its sessions stop working: an administrator's decision.
    Suspended,
}

impl AccountStatus {
    /// The status as the stores spell it: `active` or `suspended`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
        }
    }

    /// The status a store spells `name`, if any.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Active, Self::Suspended]
            .into_iter()
            .find(|status| status.as_str() == name)
    }
}

/// An account as it is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRecord {
    /// The account's id.
    pub id: AccountId,
    /// The address, as it was signed up with.
    pub email: String,
    /// The password's hash.
    pub password_hash: PasswordHash,
    /// When the address was verified, if it was.
    pub email_verified_at: Option<OffsetDateTime>,
    /// The language code of its emails and interface.
    pub language: String,
    /// Whether it may sign in.
    pub status: AccountStatus,
    /// Its plan: `free` in V1.
    pub plan: String,
    /// The bytes of its documents.
    pub storage_used_bytes: u64,
    /// When it signed up.
    pub created_at: OffsetDateTime,
}

/// A new account: active, on the free plan, with no document yet.
#[derive(Clone, Debug)]
pub struct NewAccount {
    /// Its id.
    pub id: AccountId,
    /// Its address.
    pub email: EmailAddress,
    /// Its password's hash.
    pub password_hash: PasswordHash,
    /// Its language.
    pub language: Language,
    /// When it signs up.
    pub created_at: OffsetDateTime,
}

/// Why a store of the accounts failed.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum AccountStoreError {
    /// Another account has the address, whatever its case.
    #[error("the address is taken")]
    EmailTaken,
    /// The storage failed; the detail is for the logs, and never holds an address.
    #[error("accounts storage unavailable: {0}")]
    Unavailable(String),
}

/// Where accounts are kept. Addresses are compared whatever their case.
#[async_trait]
pub trait AccountStore: Send + Sync {
    /// Adds `account`; [`AccountStoreError::EmailTaken`] when another account has its address.
    async fn create(&self, account: NewAccount) -> Result<AccountRecord, AccountStoreError>;
    /// The account `id`, if it exists.
    async fn get(&self, id: AccountId) -> Result<Option<AccountRecord>, AccountStoreError>;
    /// The account of `email`, if one has it.
    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<AccountRecord>, AccountStoreError>;
    /// Replaces the password hash of the account `id`.
    async fn set_password_hash(
        &self,
        id: AccountId,
        hash: &PasswordHash,
    ) -> Result<(), AccountStoreError>;
    /// Records that the address of the account `id` was verified `at`, unless it already was.
    async fn mark_email_verified(
        &self,
        id: AccountId,
        at: OffsetDateTime,
    ) -> Result<(), AccountStoreError>;
    /// Sets the language of the account `id`.
    async fn set_language(
        &self,
        id: AccountId,
        language: &Language,
    ) -> Result<(), AccountStoreError>;
    /// Deletes the account `id`; its sessions, tokens and library rows go with it.
    async fn delete(&self, id: AccountId) -> Result<(), AccountStoreError>;
}
