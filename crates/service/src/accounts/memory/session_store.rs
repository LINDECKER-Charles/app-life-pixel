//! Sessions in memory.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::accounts::ports::{AccountStoreError, SessionExtension, SessionRecord, SessionStore};
use crate::accounts::values::TokenHash;
use crate::ids::AccountId;

/// Sessions in memory, by the hash of their token.
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<TokenHash, SessionRecord>>,
}

impl InMemorySessionStore {
    /// A store with no session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The sessions of `account`, for tests to inspect.
    #[must_use]
    pub fn sessions_of(&self, account: AccountId) -> Vec<SessionRecord> {
        let sessions = self.lock();
        let mine = sessions
            .values()
            .filter(|session| session.account_id == account);
        mine.cloned().collect()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<TokenHash, SessionRecord>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, session: SessionRecord) -> Result<(), AccountStoreError> {
        self.lock().insert(session.token_hash, session);
        Ok(())
    }

    async fn find(
        &self,
        token_hash: &TokenHash,
    ) -> Result<Option<SessionRecord>, AccountStoreError> {
        Ok(self.lock().get(token_hash).cloned())
    }

    async fn extend(
        &self,
        token_hash: &TokenHash,
        extension: SessionExtension,
    ) -> Result<(), AccountStoreError> {
        if let Some(session) = self.lock().get_mut(token_hash) {
            session.last_seen_at = extension.seen_at;
            session.expires_at = extension.expires_at;
        }
        Ok(())
    }

    async fn delete(&self, token_hash: &TokenHash) -> Result<(), AccountStoreError> {
        self.lock().remove(token_hash);
        Ok(())
    }

    async fn delete_all(&self, account: AccountId) -> Result<(), AccountStoreError> {
        self.lock()
            .retain(|_, session| session.account_id != account);
        Ok(())
    }

    async fn delete_others(
        &self,
        account: AccountId,
        kept: &TokenHash,
    ) -> Result<(), AccountStoreError> {
        let is_kept =
            |session: &SessionRecord| session.account_id != account || session.token_hash == *kept;
        self.lock().retain(|_, session| is_kept(session));
        Ok(())
    }

    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError> {
        let mut sessions = self.lock();
        let before = sessions.len();
        sessions.retain(|_, session| session.expires_at > now);
        Ok(u64::try_from(before - sessions.len()).unwrap_or(u64::MAX))
    }
}
