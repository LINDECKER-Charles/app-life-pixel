//! Where sessions are kept, by the hash of their cookie's token.

use async_trait::async_trait;
use time::OffsetDateTime;

use super::AccountStoreError;
use crate::accounts::values::TokenHash;
use crate::ids::AccountId;

/// A session as it is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecord {
    /// The SHA-256 of the cookie's token.
    pub token_hash: TokenHash,
    /// The account signed in.
    pub account_id: AccountId,
    /// When it opened.
    pub created_at: OffsetDateTime,
    /// When its expiry last moved.
    pub last_seen_at: OffsetDateTime,
    /// When it ends, unless it is seen again.
    pub expires_at: OffsetDateTime,
}

/// A session seen again: when, and its new expiry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionExtension {
    /// When it was seen.
    pub seen_at: OffsetDateTime,
    /// When it now ends, unless it is seen again.
    pub expires_at: OffsetDateTime,
}

/// Where sessions are kept.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Adds `session`.
    async fn create(&self, session: SessionRecord) -> Result<(), AccountStoreError>;
    /// The session of `token_hash`, expired or not, if it exists.
    async fn find(
        &self,
        token_hash: &TokenHash,
    ) -> Result<Option<SessionRecord>, AccountStoreError>;
    /// Sets the `last_seen_at` and `expires_at` of the session of `token_hash`.
    async fn extend(
        &self,
        token_hash: &TokenHash,
        extension: SessionExtension,
    ) -> Result<(), AccountStoreError>;
    /// Deletes the session of `token_hash`, if it exists.
    async fn delete(&self, token_hash: &TokenHash) -> Result<(), AccountStoreError>;
    /// Deletes every session of `account`.
    async fn delete_all(&self, account: AccountId) -> Result<(), AccountStoreError>;
    /// Deletes every session of `account` but the one of `kept`.
    async fn delete_others(
        &self,
        account: AccountId,
        kept: &TokenHash,
    ) -> Result<(), AccountStoreError>;
    /// Deletes the sessions expired at `now`, and counts them.
    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError>;
}
