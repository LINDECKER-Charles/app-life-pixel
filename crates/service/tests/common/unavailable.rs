//! A store whose every operation fails, as a database that went away.

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use life_pixel_service::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, LibraryStore, NewAnimationRecord,
    ProjectRecord, StoreError,
};
use life_pixel_service::{AnimationId, Owner, Page, PageRequest, ProjectId};
use time::OffsetDateTime;

/// Fails every operation with [`StoreError::Unavailable`].
pub struct UnavailableStore;

fn unavailable<T>() -> Result<T, StoreError> {
    Err(StoreError::Unavailable("connection refused".to_owned()))
}

#[async_trait]
impl LibraryStore for UnavailableStore {
    async fn create_project(&self, _: &Owner, _: ProjectRecord) -> Result<(), StoreError> {
        unavailable()
    }

    async fn get_project(&self, _: &Owner, _: ProjectId) -> Result<ProjectRecord, StoreError> {
        unavailable()
    }

    async fn list_projects(
        &self,
        _: &Owner,
        _: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        unavailable()
    }

    async fn rename_project(
        &self,
        _: &Owner,
        _: ProjectId,
        _: Name,
        _: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError> {
        unavailable()
    }

    async fn delete_project(&self, _: &Owner, _: ProjectId) -> Result<(), StoreError> {
        unavailable()
    }

    async fn create_animation(
        &self,
        _: &Owner,
        _: NewAnimationRecord,
        _: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        unavailable()
    }

    async fn get_animation(
        &self,
        _: &Owner,
        _: AnimationId,
    ) -> Result<AnimationRecord, StoreError> {
        unavailable()
    }

    async fn list_animations(
        &self,
        _: &Owner,
        _: AnimationFilter,
        _: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        unavailable()
    }

    async fn read_document(
        &self,
        _: &Owner,
        _: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        unavailable()
    }

    async fn write_document(
        &self,
        _: &Owner,
        _: DocumentWrite,
        _: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        unavailable()
    }

    async fn move_animation(
        &self,
        _: &Owner,
        _: AnimationId,
        _: ProjectId,
        _: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError> {
        unavailable()
    }

    async fn delete_animation(&self, _: &Owner, _: AnimationId) -> Result<(), StoreError> {
        unavailable()
    }

    async fn usage(&self, _: &Owner) -> Result<u64, StoreError> {
        unavailable()
    }

    async fn delete_everything(&self, _: &Owner) -> Result<(), StoreError> {
        unavailable()
    }
}
