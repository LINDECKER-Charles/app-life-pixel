//! A change saves with the version it read: on a conflict it reads again and retries once.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use life_pixel_service::memory::InMemoryLibraryStore;
use life_pixel_service::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, LibraryStore, NewAnimationRecord,
    ProjectRecord, StoreError,
};
use life_pixel_service::{AnimationId, Owner, Page, PageRequest, ProjectId};
use serde_json::json;
use time::OffsetDateTime;

use super::support::{Fixture, assert_coded};

/// A store where someone else saves the document right before each of the next `interruptions`
/// writes, which then conflict.
struct InterruptedStore {
    inner: InMemoryLibraryStore,
    interruptions: AtomicU32,
    writes: AtomicU32,
}

impl InterruptedStore {
    fn new() -> Self {
        Self {
            inner: InMemoryLibraryStore::new(),
            interruptions: AtomicU32::new(0),
            writes: AtomicU32::new(0),
        }
    }

    /// Saves the stored document again, as another client would: its version changes.
    async fn interrupt(&self, owner: &Owner, write: &DocumentWrite) -> Result<(), StoreError> {
        let (record, document) = self.inner.read_document(owner, write.id).await?;
        let other = DocumentWrite {
            id: write.id,
            expected_version: record.version,
            meta: record.meta,
            document,
            at: write.at,
        };
        self.inner
            .write_document(owner, other, None)
            .await
            .map(|_| ())
    }
}

#[async_trait]
impl LibraryStore for InterruptedStore {
    async fn create_project(
        &self,
        owner: &Owner,
        project: ProjectRecord,
    ) -> Result<(), StoreError> {
        self.inner.create_project(owner, project).await
    }

    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        self.inner.get_project(owner, id).await
    }

    async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        self.inner.list_projects(owner, page).await
    }

    async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: Name,
        at: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError> {
        self.inner.rename_project(owner, id, name, at).await
    }

    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError> {
        self.inner.delete_project(owner, id).await
    }

    async fn create_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.inner.create_animation(owner, new, quota).await
    }

    async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, StoreError> {
        self.inner.get_animation(owner, id).await
    }

    async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        self.inner.list_animations(owner, filter, page).await
    }

    async fn read_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        self.inner.read_document(owner, id).await
    }

    async fn write_document(
        &self,
        owner: &Owner,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        let left = self.interruptions.load(Ordering::SeqCst);
        if left > 0 {
            self.interruptions.store(left - 1, Ordering::SeqCst);
            self.interrupt(owner, &write).await?;
        }
        self.inner.write_document(owner, write, quota).await
    }

    async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
        at: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError> {
        self.inner.move_animation(owner, id, to, at).await
    }

    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError> {
        self.inner.delete_animation(owner, id).await
    }

    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError> {
        self.inner.usage(owner).await
    }

    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError> {
        self.inner.delete_everything(owner).await
    }
}

async fn interrupted(interruptions: u32) -> (Fixture, Arc<InterruptedStore>) {
    let store = Arc::new(InterruptedStore::new());
    let fixture = Fixture::over(store.clone()).await;
    store.interruptions.store(interruptions, Ordering::SeqCst);
    (fixture, store)
}

#[tokio::test]
async fn a_conflict_is_read_again_and_retried_once() {
    let (fixture, store) = interrupted(1).await;

    let view = fixture
        .write(0, &["1...", "....", "....", "...2"])
        .await
        .unwrap();

    assert_eq!(store.writes.load(Ordering::SeqCst), 2);
    assert_eq!(view.version, fixture.version().await);
    let rows = &fixture.describe(&[0]).await.grids[0].rows;
    assert_eq!(rows, &["1...", "....", "....", "...2"]);
}

#[tokio::test]
async fn a_second_conflict_is_returned_with_the_current_version() {
    let (fixture, store) = interrupted(2).await;

    let error = fixture.write(0, &["1..."; 4]).await.unwrap_err();

    assert_eq!(store.writes.load(Ordering::SeqCst), 2);
    let current = fixture.version().await;
    assert_coded(
        &error,
        "document.version_conflict",
        json!({ "current": current }),
    );
    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, ["...."; 4]);
}
