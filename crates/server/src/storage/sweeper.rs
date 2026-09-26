//! The sweeper: deletes the documents no row references — left by a write whose transaction
//! failed and whose cleanup failed too, or by a crash between the two —, once they are old enough
//! that no write in progress can still point a row at them. It does the same for the support
//! screenshots whose request is gone (H9).

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::TryStreamExt;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt};
use sqlx::PgPool;
use thiserror::Error;

use super::keys::{DOCUMENTS_PREFIX, SUPPORT_PREFIX};
use super::metrics::{count_orphans_deleted, timed};

/// How often the sweeper runs.
pub const SWEEP_PERIOD: Duration = Duration::from_secs(6 * 60 * 60);
/// How old an unreferenced object must be to be deleted.
pub const ORPHAN_MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// How many keys the sweeper checks against the index at once.
pub const SWEEP_BATCH_KEYS: usize = 1_000;

/// A prefix the sweeper lists, and the query of which of a batch of its keys a row references.
struct Swept {
    prefix: &'static str,
    referenced: &'static str,
    query_name: &'static str,
}

/// Every prefix the sweeper lists: one line per prefix.
const SWEPT: [Swept; 2] = [
    Swept {
        prefix: DOCUMENTS_PREFIX,
        referenced: "select document_key from animations where document_key = any($1)",
        query_name: "referenced_document_keys",
    },
    Swept {
        prefix: SUPPORT_PREFIX,
        referenced: "select screenshot_key from support_requests where screenshot_key = any($1)",
        query_name: "referenced_screenshot_keys",
    },
];

/// Why a sweep stopped.
#[derive(Debug, Error)]
pub enum SweepError {
    /// The objects could not be listed.
    #[error("the objects cannot be listed: {0}")]
    List(#[from] object_store::Error),
    /// The index could not be read.
    #[error("the index cannot be read: {0}")]
    Database(#[from] sqlx::Error),
}

/// Deletes the old objects under `documents/` and `support/` that no row references.
#[derive(Clone)]
pub struct Sweeper {
    pool: PgPool,
    objects: Arc<dyn ObjectStore>,
    min_age: Duration,
}

impl Sweeper {
    /// The sweeper of the index of `pool` and the documents of `objects`, for objects older than
    /// [`ORPHAN_MIN_AGE`].
    #[must_use]
    pub fn new(pool: PgPool, objects: Arc<dyn ObjectStore>) -> Self {
        Self {
            pool,
            objects,
            min_age: ORPHAN_MIN_AGE,
        }
    }

    /// The sweeper deleting unreferenced objects older than `min_age`: for tests.
    #[must_use]
    pub fn with_min_age(self, min_age: Duration) -> Self {
        Self { min_age, ..self }
    }

    /// Deletes the old unreferenced objects, by batches of [`SWEEP_BATCH_KEYS`] keys; returns
    /// how many it deleted. A deletion that fails is logged, and retried at the next sweep.
    ///
    /// # Errors
    ///
    /// When the objects cannot be listed or the index cannot be read.
    pub async fn sweep(&self) -> Result<u64, SweepError> {
        let mut deleted = 0;
        for swept in &SWEPT {
            deleted += self.sweep_prefix(swept).await?;
        }
        Ok(deleted)
    }

    /// Deletes the old unreferenced objects under the prefix of `swept`; returns how many.
    async fn sweep_prefix(&self, swept: &Swept) -> Result<u64, SweepError> {
        let cutoff = cutoff_millis(self.min_age);
        let prefix = Path::from(swept.prefix);
        let mut listing = self.objects.list(Some(&prefix));
        let (mut batch, mut deleted) = (Vec::with_capacity(SWEEP_BATCH_KEYS), 0);
        while let Some(object) = listing.try_next().await? {
            if object.last_modified.timestamp_millis() > cutoff {
                continue;
            }
            batch.push(object.location);
            if batch.len() == SWEEP_BATCH_KEYS {
                deleted += self.sweep_batch(swept, std::mem::take(&mut batch)).await?;
            }
        }
        deleted += self.sweep_batch(swept, batch).await?;
        Ok(deleted)
    }

    /// Deletes the keys of `batch` no row references; returns how many it deleted.
    async fn sweep_batch(&self, swept: &Swept, batch: Vec<Path>) -> Result<u64, SweepError> {
        if batch.is_empty() {
            return Ok(0);
        }
        let referenced = self.referenced(swept, &batch).await?;
        let mut deleted = 0;
        for key in batch
            .iter()
            .filter(|key| !referenced.contains(key.as_ref()))
        {
            deleted += u64::from(self.delete_orphan(key).await);
        }
        count_orphans_deleted(deleted);
        Ok(deleted)
    }

    /// The keys of `batch` a row references.
    async fn referenced(
        &self,
        swept: &Swept,
        batch: &[Path],
    ) -> Result<HashSet<String>, sqlx::Error> {
        let keys: Vec<String> = batch.iter().map(ToString::to_string).collect();
        let query = sqlx::query_scalar::<_, String>(swept.referenced).bind(&keys);
        let referenced = timed(swept.query_name, query.fetch_all(&self.pool)).await?;
        Ok(referenced.into_iter().collect())
    }

    /// Deletes the orphan at `key`; whether it did. A failure is logged.
    async fn delete_orphan(&self, key: &Path) -> bool {
        let deleted = self.objects.delete(key).await;
        if let Err(error) = &deleted {
            tracing::warn!(%error, %key, "an orphan could not be deleted");
        }
        deleted.is_ok()
    }
}

/// The time, in milliseconds since the epoch, before which an object is old enough.
fn cutoff_millis(min_age: Duration) -> i64 {
    let cutoff = SystemTime::now().checked_sub(min_age).unwrap_or(UNIX_EPOCH);
    let since_epoch = cutoff.duration_since(UNIX_EPOCH).unwrap_or_default();
    i64::try_from(since_epoch.as_millis()).unwrap_or(i64::MAX)
}
