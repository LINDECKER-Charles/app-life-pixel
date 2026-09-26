//! An in-memory adapter of the token port, for tests: feature `testing`.

use std::cmp::Reverse;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use super::ports::{AccessTokenRecord, FoundToken, TokenStore, TokenStoreError};
use crate::accounts::TokenHash;
use crate::accounts::ports::AccountStatus;
use crate::ids::AccountId;

/// Tokens in memory, with the suspended accounts a test sets.
#[derive(Debug, Default)]
pub struct InMemoryTokenStore {
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    tokens: Vec<(AccessTokenRecord, TokenHash)>,
    suspended: Vec<AccountId>,
}

impl InMemoryTokenStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Suspends `account`: its tokens are found with that status.
    pub fn suspend(&self, account: AccountId) {
        self.state().suspended.push(account);
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn create(
        &self,
        token: &AccessTokenRecord,
        (hash, max_active): (&TokenHash, usize),
    ) -> Result<(), TokenStoreError> {
        let mut state = self.state();
        let active = state
            .tokens
            .iter()
            .filter(|(record, _)| record.account == token.account)
            .filter(|(record, _)| record.is_active(token.created_at))
            .count();
        if active >= max_active {
            return Err(TokenStoreError::LimitReached);
        }
        state.tokens.push((token.clone(), *hash));
        Ok(())
    }

    async fn list(
        &self,
        account: AccountId,
        active_at: Option<OffsetDateTime>,
    ) -> Result<Vec<AccessTokenRecord>, TokenStoreError> {
        let state = self.state();
        let mut tokens: Vec<AccessTokenRecord> = state
            .tokens
            .iter()
            .map(|(record, _)| record)
            .filter(|record| record.account == account)
            .filter(|record| active_at.is_none_or(|now| record.is_active(now)))
            .cloned()
            .collect();
        tokens.sort_by_key(|token| Reverse((token.created_at, token.id)));
        Ok(tokens)
    }

    async fn revoke(
        &self,
        (account, id): (AccountId, Uuid),
        at: OffsetDateTime,
    ) -> Result<bool, TokenStoreError> {
        let mut state = self.state();
        let found = state.tokens.iter_mut().find(|(record, _)| {
            record.id == id && record.account == account && record.is_active(at)
        });
        let Some((record, _)) = found else {
            return Ok(false);
        };
        record.revoked_at = Some(at);
        Ok(true)
    }

    async fn find(&self, hash: &TokenHash) -> Result<Option<FoundToken>, TokenStoreError> {
        let state = self.state();
        let found = state.tokens.iter().find(|(_, stored)| stored == hash);
        Ok(found.map(|(record, _)| FoundToken {
            record: record.clone(),
            account_status: if state.suspended.contains(&record.account) {
                AccountStatus::Suspended
            } else {
                AccountStatus::Active
            },
        }))
    }

    async fn touch(&self, id: Uuid, at: OffsetDateTime) -> Result<(), TokenStoreError> {
        let mut state = self.state();
        let found = state.tokens.iter_mut().find(|(record, _)| record.id == id);
        if let Some((record, _)) = found {
            record.last_used_at = Some(at);
        }
        Ok(())
    }
}
