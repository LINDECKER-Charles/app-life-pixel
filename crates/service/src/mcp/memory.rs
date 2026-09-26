//! In-memory adapters of the MCP endpoint's ports, for tests: feature `testing`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use time::Date;

use super::McpStores;
use super::ports::{AnimationOwners, McpStoreError, McpUsageStore};
use crate::ids::{AccountId, AnimationId};

/// The calls of each account per day, in memory.
#[derive(Debug, Default)]
pub struct InMemoryMcpUsage {
    calls: Mutex<HashMap<(AccountId, Date), u32>>,
}

impl InMemoryMcpUsage {
    /// No call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The calls counted for `account` on `day`.
    #[must_use]
    pub fn calls(&self, account: AccountId, day: Date) -> u32 {
        let calls = self.calls.lock().unwrap_or_else(PoisonError::into_inner);
        calls.get(&(account, day)).copied().unwrap_or_default()
    }
}

#[async_trait]
impl McpUsageStore for InMemoryMcpUsage {
    async fn add_call(
        &self,
        account: AccountId,
        (day, limit): (Date, u32),
    ) -> Result<bool, McpStoreError> {
        let mut calls = self.calls.lock().unwrap_or_else(PoisonError::into_inner);
        let count = calls.entry((account, day)).or_default();
        if *count >= limit {
            return Ok(false);
        }
        *count += 1;
        Ok(true)
    }
}

/// The owners of the animations a test names.
#[derive(Debug, Default)]
pub struct InMemoryAnimationOwners {
    owners: Mutex<HashMap<AnimationId, AccountId>>,
}

impl InMemoryAnimationOwners {
    /// No animation known.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that `animation` belongs to `account`.
    pub fn insert(&self, animation: AnimationId, account: AccountId) {
        let mut owners = self.owners.lock().unwrap_or_else(PoisonError::into_inner);
        owners.insert(animation, account);
    }
}

#[async_trait]
impl AnimationOwners for InMemoryAnimationOwners {
    async fn owner_of(&self, id: AnimationId) -> Result<Option<AccountId>, McpStoreError> {
        let owners = self.owners.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(owners.get(&id).copied())
    }
}

/// MCP stores in memory, for the tests that need them without looking inside.
#[must_use]
pub fn in_memory_stores() -> McpStores {
    McpStores {
        usage: Arc::new(InMemoryMcpUsage::new()),
        owners: Arc::new(InMemoryAnimationOwners::new()),
    }
}
