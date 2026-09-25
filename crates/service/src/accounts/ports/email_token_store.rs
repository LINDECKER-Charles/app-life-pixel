//! Where the tokens of emailed links are kept: single use, until they expire.

use async_trait::async_trait;
use time::OffsetDateTime;

use super::AccountStoreError;
use crate::accounts::values::TokenHash;
use crate::ids::AccountId;

/// What an emailed token is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenPurpose {
    /// Verifying the account's address: 7 days.
    VerifyEmail,
    /// Setting a new password: one hour.
    ResetPassword,
}

impl TokenPurpose {
    /// The purpose as the stores spell it: `verify_email` or `reset_password`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VerifyEmail => "verify_email",
            Self::ResetPassword => "reset_password",
        }
    }
}

/// An emailed token as it is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmailTokenRecord {
    /// The SHA-256 of the token.
    pub token_hash: TokenHash,
    /// The account it acts for.
    pub account_id: AccountId,
    /// What it is for.
    pub purpose: TokenPurpose,
    /// When it was made.
    pub created_at: OffsetDateTime,
    /// When it stops working.
    pub expires_at: OffsetDateTime,
}

/// A client's use of an emailed token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenUse {
    /// The SHA-256 of the token the client sent.
    pub token_hash: TokenHash,
    /// What the client uses it for.
    pub purpose: TokenPurpose,
    /// When.
    pub at: OffsetDateTime,
}

/// Where the tokens of emailed links are kept.
#[async_trait]
pub trait EmailTokenStore: Send + Sync {
    /// Adds `token`, unused.
    async fn create(&self, token: EmailTokenRecord) -> Result<(), AccountStoreError>;
    /// Marks the token of `used.token_hash` used at `used.at`, and returns its account, when it
    /// exists for `used.purpose`, is unused and expires after `used.at`; `None` otherwise. Of two
    /// calls on one token, one at most succeeds.
    async fn consume(&self, used: TokenUse) -> Result<Option<AccountId>, AccountStoreError>;
    /// Deletes the tokens expired at `now`, used or not, and counts them.
    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError>;
}
