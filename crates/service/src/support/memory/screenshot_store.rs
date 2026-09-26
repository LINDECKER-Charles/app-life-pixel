//! Screenshots kept in memory.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use bytes::Bytes;

use crate::support::SupportRequestId;
use crate::support::ports::{ScreenshotStore, SupportStoreError};

/// Keeps screenshots by key, `support/<request>/screenshot.png` as the server's.
#[derive(Debug, Default)]
pub struct InMemoryScreenshotStore {
    objects: Mutex<BTreeMap<String, Bytes>>,
}

impl InMemoryScreenshotStore {
    /// A store with no screenshot yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The keys stored, sorted.
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.lock().keys().cloned().collect()
    }

    fn lock(&self) -> MutexGuard<'_, BTreeMap<String, Bytes>> {
        self.objects.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl ScreenshotStore for InMemoryScreenshotStore {
    async fn put(
        &self,
        request: SupportRequestId,
        png: Bytes,
    ) -> Result<String, SupportStoreError> {
        let key = format!("support/{request}/screenshot.png");
        self.lock().insert(key.clone(), png);
        Ok(key)
    }

    async fn read(&self, key: &str) -> Result<Bytes, SupportStoreError> {
        let missing = || SupportStoreError(format!("no screenshot at {key}"));
        self.lock().get(key).cloned().ok_or_else(missing)
    }

    async fn delete(&self, key: &str) -> Result<(), SupportStoreError> {
        self.lock().remove(key);
        Ok(())
    }
}
