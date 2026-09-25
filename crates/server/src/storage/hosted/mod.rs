//! `HostedLibraryStore`: the library of `Owner::Account`, its index in Postgres and its
//! documents in object storage.
//!
//! A document is written in three steps: the object first, at a new key, outside any
//! transaction; then one transaction that locks the account's row and the animation's, checks the
//! version and the quota, points the row at the new key and moves the usage; then the old object
//! is deleted. When the transaction fails, the new object is deleted instead. A deletion that
//! fails is logged, and the [`Sweeper`](crate::storage::Sweeper) collects the object later: usage
//! is always the bytes the rows point at.

mod account;
mod animations;
mod documents;
mod paging;
mod projects;
mod removal;
mod rows;

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use life_pixel_service::paging::{Page, PageRequest};
use life_pixel_service::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, LibraryStore, NewAnimationRecord,
    ProjectRecord, StoreError,
};
use life_pixel_service::{AccountId, AnimationId, Owner, ProjectId};
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

use self::account::account_of;
use self::paging::PagedRow;
use self::rows::{database, objects};

/// The hosted library: the index of `pool`, the documents of `objects`.
#[derive(Clone)]
pub struct HostedLibraryStore {
    pool: PgPool,
    objects: Arc<dyn ObjectStore>,
}

impl HostedLibraryStore {
    /// The store over the migrated database of `pool` and the object storage `objects`.
    #[must_use]
    pub fn new(pool: PgPool, objects: Arc<dyn ObjectStore>) -> Self {
        Self { pool, objects }
    }

    /// A new transaction.
    async fn begin(&self) -> Result<Transaction<'static, Postgres>, StoreError> {
        self.pool.begin().await.map_err(database)
    }

    /// Stores `document` at `key`.
    async fn put_object(&self, key: &Path, document: Bytes) -> Result<(), StoreError> {
        let payload = PutPayload::from_bytes(document);
        self.objects.put(key, payload).await.map_err(objects)?;
        Ok(())
    }

    /// Deletes the objects at `keys`; a failure is logged, and left to the sweeper.
    async fn delete_objects(&self, keys: impl IntoIterator<Item = Path>) {
        for key in keys {
            self.delete_object(&key).await;
        }
    }

    /// Deletes the object at `key`; a failure is logged, and left to the sweeper.
    async fn delete_object(&self, key: &Path) {
        if let Err(error) = self.objects.delete(key).await {
            tracing::warn!(%error, %key, "a document could not be deleted: the sweeper will");
        }
    }
}

/// A new document, stored at `key` and not yet recorded, and the quota its account is under.
#[derive(Clone, Copy)]
struct StagedDocument<'a> {
    account: AccountId,
    key: &'a Path,
    quota: Option<u64>,
}

#[async_trait]
impl LibraryStore for HostedLibraryStore {
    async fn create_project(
        &self,
        owner: &Owner,
        project: ProjectRecord,
    ) -> Result<(), StoreError> {
        self.insert_project(account_of(owner)?, project).await
    }

    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        self.select_project(account_of(owner)?, id).await
    }

    async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        self.select_projects(account_of(owner)?, page).await
    }

    async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: Name,
        at: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError> {
        self.update_project_name((account_of(owner)?, id), (name, at))
            .await
    }

    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError> {
        self.remove_project(account_of(owner)?, id).await
    }

    async fn create_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.insert_animation(account_of(owner)?, (new, quota))
            .await
    }

    async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, StoreError> {
        self.select_animation(account_of(owner)?, id)
            .await?
            .into_record()
    }

    async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        self.select_animations(account_of(owner)?, (&filter, &page))
            .await
    }

    async fn read_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        self.read_object(account_of(owner)?, id).await
    }

    async fn write_document(
        &self,
        owner: &Owner,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.replace_object(account_of(owner)?, (write, quota))
            .await
    }

    async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
        at: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError> {
        self.update_animation_project((account_of(owner)?, id), (to, at))
            .await
    }

    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError> {
        self.remove_animation(account_of(owner)?, id).await
    }

    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError> {
        self.select_usage(account_of(owner)?).await
    }

    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError> {
        self.remove_everything(account_of(owner)?).await
    }
}
