//! Documents: read one back, and replace one in three steps — the new object, the transaction,
//! the old object deleted.

use bytes::Bytes;
use life_pixel_service::ports::library_store::{AnimationRecord, DocumentWrite, StoreError};
use life_pixel_service::quota::StorageChange;
use life_pixel_service::{AccountId, AnimationId};
use object_store::ObjectStoreExt;
use object_store::path::Path;
use sqlx::PgConnection;

use super::account::{add_usage, lock_usage, usage_delta};
use super::animations::lock_animation;
use super::paging::PagedRow;
use super::rows::{AnimationRow, animation_columns, bigint, database, objects};
use super::{HostedLibraryStore, StagedDocument};
use crate::storage::keys::new_document_key;
use crate::storage::metrics::timed;

const WRITE_DOCUMENT: &str = concat!(
    "update animations set title = $2, width = $3, height = $4, frame_count = $5, \
     document_key = $6, document_bytes = $7, version = version + 1, updated_at = $8 \
     where id = $1 returning ",
    animation_columns!()
);
/// How many times a read follows a document that a write replaced meanwhile.
const READ_ATTEMPTS: usize = 3;

impl HostedLibraryStore {
    /// The animation's record and its document. A write may replace the document between the two
    /// reads, and delete the object read: the read then starts again from the row.
    pub(super) async fn read_object(
        &self,
        account: AccountId,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        for _ in 0..READ_ATTEMPTS {
            let row = self.select_animation(account, id).await?;
            let key = Path::from(row.document_key.as_str());
            match self.objects.get(&key).await {
                Ok(object) => {
                    let bytes = object.bytes().await.map_err(objects)?;
                    return Ok((row.into_record()?, bytes));
                }
                Err(object_store::Error::NotFound { .. }) => {}
                Err(error) => return Err(objects(error)),
            }
        }
        let missing = format!("the document of animation {id} is missing");
        Err(StoreError::Unavailable(missing))
    }

    /// Stores the new document, records it, then deletes the old one; when the record fails, the
    /// new document is deleted instead.
    pub(super) async fn replace_object(
        &self,
        account: AccountId,
        (write, quota): (DocumentWrite, Option<u64>),
    ) -> Result<AnimationRecord, StoreError> {
        let key = new_document_key(account, write.id);
        self.put_object(&key, write.document.clone()).await?;
        let staged = StagedDocument {
            account,
            key: &key,
            quota,
        };
        match self.record_document(&staged, &write).await {
            Ok((record, old_key)) => {
                self.delete_objects([old_key]).await;
                Ok(record)
            }
            Err(error) => {
                self.delete_objects([key]).await;
                Err(error)
            }
        }
    }

    /// The transaction of a write: locks the account and the animation, checks the version and
    /// the quota, points the row at the new key and moves the usage. Returns the new record and
    /// the key the row pointed at.
    async fn record_document(
        &self,
        staged: &StagedDocument<'_>,
        write: &DocumentWrite,
    ) -> Result<(AnimationRecord, Path), StoreError> {
        let mut transaction = self.begin().await?;
        let used = lock_usage(&mut transaction, staged.account).await?;
        let used = used.ok_or(StoreError::AnimationNotFound)?;
        let current = lock_animation(&mut transaction, staged.account, write.id).await?;
        let old_key = Path::from(current.document_key.as_str());
        let current = current.into_record()?;
        if current.version != write.expected_version {
            return Err(StoreError::VersionConflict {
                current: current.version,
            });
        }
        let added = u64::try_from(write.document.len()).unwrap_or(u64::MAX);
        let freed = current.document_bytes;
        let change = StorageChange { used, freed, added };
        change.check(staged.quota)?;
        let row = update_row(&mut transaction, staged.key, write).await?;
        add_usage(&mut transaction, staged.account, usage_delta(&change)?).await?;
        transaction.commit().await.map_err(database)?;
        Ok((row.into_record()?, old_key))
    }
}

/// Points the animation's row at the document of `key`, one version further.
async fn update_row(
    connection: &mut PgConnection,
    key: &Path,
    write: &DocumentWrite,
) -> Result<AnimationRow, StoreError> {
    let bytes = u64::try_from(write.document.len()).unwrap_or(u64::MAX);
    let query = sqlx::query_as::<_, AnimationRow>(WRITE_DOCUMENT)
        .bind(write.id.uuid())
        .bind(write.meta.title.as_str())
        .bind(i32::from(write.meta.width))
        .bind(i32::from(write.meta.height))
        .bind(i32::from(write.meta.frame_count))
        .bind(key.as_ref())
        .bind(bigint(bytes)?)
        .bind(write.at);
    let row = timed("write_document", query.fetch_one(connection)).await;
    row.map_err(database)
}
