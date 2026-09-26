//! Where personal access tokens are kept: their record, and the hash of their secret.

use async_trait::async_trait;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use super::values::TokenScope;
use crate::accounts::TokenHash;
use crate::accounts::ports::AccountStatus;
use crate::ids::AccountId;

/// A token as it is stored, without the hash of its secret: nothing that reads a record can
/// leak what authenticates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessTokenRecord {
    /// The token's id.
    pub id: Uuid,
    /// The account it acts for.
    pub account: AccountId,
    /// The name its owner gave it.
    pub name: String,
    /// `lp_pat_` and the first 4 random characters, for display.
    pub prefix: String,
    /// What it grants, in the order of [`TokenScope::ALL`].
    pub scopes: Vec<TokenScope>,
    /// When it was created.
    pub created_at: OffsetDateTime,
    /// When it stops working.
    pub expires_at: OffsetDateTime,
    /// When it was last used, to the minute; `None` before its first use.
    pub last_used_at: Option<OffsetDateTime>,
    /// When its owner revoked it, if they did.
    pub revoked_at: Option<OffsetDateTime>,
}

impl AccessTokenRecord {
    /// Whether the token still works at `now`: neither revoked nor expired.
    #[must_use]
    pub fn is_active(&self, now: OffsetDateTime) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}

/// A token found by the hash of its secret, with the status of its account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundToken {
    /// The token.
    pub record: AccessTokenRecord,
    /// Whether its account may act.
    pub account_status: AccountStatus,
}

/// Why a token store failed.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum TokenStoreError {
    /// The account already has the most active tokens it may.
    #[error("too many active tokens")]
    LimitReached,
    /// The storage failed; the detail is for the logs, and never holds a secret.
    #[error("token storage unavailable: {0}")]
    Unavailable(String),
}

/// Where personal access tokens are kept.
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Adds `token`, whose secret hashes to `hash`, unless its account already has `max_active`
    /// tokens active at its creation: [`TokenStoreError::LimitReached`]. Counting and adding are
    /// one change.
    async fn create(
        &self,
        token: &AccessTokenRecord,
        (hash, max_active): (&TokenHash, usize),
    ) -> Result<(), TokenStoreError>;
    /// The tokens of `account`, the most recently created first; only those active at
    /// `active_at` when it is given.
    async fn list(
        &self,
        account: AccountId,
        active_at: Option<OffsetDateTime>,
    ) -> Result<Vec<AccessTokenRecord>, TokenStoreError>;
    /// Revokes the token `id` of `account` `at`: `false` when the account has no such token
    /// active at `at`.
    async fn revoke(
        &self,
        (account, id): (AccountId, Uuid),
        at: OffsetDateTime,
    ) -> Result<bool, TokenStoreError>;
    /// The token whose secret hashes to `hash`, active or not, if any.
    async fn find(&self, hash: &TokenHash) -> Result<Option<FoundToken>, TokenStoreError>;
    /// Records that the token `id` was used `at`.
    async fn touch(&self, id: Uuid, at: OffsetDateTime) -> Result<(), TokenStoreError>;
}
