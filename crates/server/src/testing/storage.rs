//! `TestStorage`: the object storage of `.env` — S3Mock's bucket on the local stack — under a
//! random prefix of its own, emptied when the test ends.

use std::sync::Arc;

use futures_util::TryStreamExt;
use object_store::prefix::PrefixStore;
use object_store::{ObjectStore, ObjectStoreExt};

use super::background::run_to_end;
use super::env::{TestSetupError, load_test_env};
use crate::config::{Config, StorageConfig};
use crate::storage::object_store;

/// The prefix of every test's objects.
const TEST_PREFIX: &str = "lp_test_";

/// The objects of one test, under a prefix of their own.
pub struct TestStorage {
    config: StorageConfig,
    prefix: String,
    objects: Arc<dyn ObjectStore>,
}

impl TestStorage {
    /// The storage of `.env` under `lp_test_<16 random hexadecimal digits>/`.
    ///
    /// # Errors
    ///
    /// When the configuration of `.env` is incomplete, or the storage cannot be set up.
    pub fn create() -> Result<Self, TestSetupError> {
        load_test_env();
        let config = Config::from_env()?.storage;
        let prefix = format!("{TEST_PREFIX}{:016x}", rand::random::<u64>());
        let objects = prefixed(&config, &prefix)?;
        Ok(Self {
            config,
            prefix,
            objects,
        })
    }

    /// The objects under the test's prefix.
    #[must_use]
    pub fn objects(&self) -> Arc<dyn ObjectStore> {
        Arc::clone(&self.objects)
    }

    /// The test's prefix, in the storage of `.env`.
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// The keys under the test's prefix, sorted.
    ///
    /// # Errors
    ///
    /// When the objects cannot be listed.
    pub async fn keys(&self) -> Result<Vec<String>, TestSetupError> {
        let listed: Vec<_> = self.objects.list(None).try_collect().await?;
        let mut keys: Vec<String> = listed
            .iter()
            .map(|meta| meta.location.to_string())
            .collect();
        keys.sort();
        Ok(keys)
    }
}

impl Drop for TestStorage {
    fn drop(&mut self) {
        let (config, prefix) = (self.config.clone(), self.prefix.clone());
        run_to_end("test storage", move || async move {
            let objects = prefixed(&config, &prefix)?;
            let listed: Vec<_> = objects.list(None).try_collect().await?;
            for meta in listed {
                objects.delete(&meta.location).await?;
            }
            Ok(())
        });
    }
}

fn prefixed(config: &StorageConfig, prefix: &str) -> Result<Arc<dyn ObjectStore>, TestSetupError> {
    let root = object_store(config)?;
    Ok(Arc::new(PrefixStore::new(root, prefix)))
}
