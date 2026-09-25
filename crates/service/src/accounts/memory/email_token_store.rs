//! Emailed tokens in memory.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::accounts::ports::{AccountStoreError, EmailTokenRecord, EmailTokenStore, TokenUse};
use crate::accounts::values::TokenHash;
use crate::ids::AccountId;

/// A token, and when it was used.
#[derive(Debug)]
struct StoredToken {
    record: EmailTokenRecord,
    used_at: Option<OffsetDateTime>,
}

/// Emailed tokens in memory, by their hash.
#[derive(Debug, Default)]
pub struct InMemoryEmailTokenStore {
    tokens: Mutex<HashMap<TokenHash, StoredToken>>,
}

impl InMemoryEmailTokenStore {
    /// A store with no token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many tokens are stored, used or not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether no token is stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<TokenHash, StoredToken>> {
        self.tokens.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl EmailTokenStore for InMemoryEmailTokenStore {
    async fn create(&self, token: EmailTokenRecord) -> Result<(), AccountStoreError> {
        let stored = StoredToken {
            record: token,
            used_at: None,
        };
        self.lock().insert(stored.record.token_hash, stored);
        Ok(())
    }

    async fn consume(&self, used: TokenUse) -> Result<Option<AccountId>, AccountStoreError> {
        let mut tokens = self.lock();
        let Some(token) = tokens.get_mut(&used.token_hash) else {
            return Ok(None);
        };
        let is_usable = token.record.purpose == used.purpose
            && token.used_at.is_none()
            && token.record.expires_at > used.at;
        if !is_usable {
            return Ok(None);
        }
        token.used_at = Some(used.at);
        Ok(Some(token.record.account_id))
    }

    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError> {
        let mut tokens = self.lock();
        let before = tokens.len();
        tokens.retain(|_, token| token.record.expires_at > now);
        Ok(u64::try_from(before - tokens.len()).unwrap_or(u64::MAX))
    }
}
