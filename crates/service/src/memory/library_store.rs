//! The library store in memory: versions count up from 1.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use time::OffsetDateTime;

use super::owner_library::{
    OwnerLibrary, StoredAnimation, animation_cursor, byte_length, project_cursor,
};
use crate::ids::{AnimationId, ProjectId};
use crate::owner::Owner;
use crate::paging::{Page, PageRequest, paginate};
use crate::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, LibraryStore, NewAnimationRecord,
    ProjectRecord, StoreError,
};

/// The version of a new document.
const FIRST_VERSION: u64 = 1;

/// Every owner's library, in memory; each operation is atomic.
#[derive(Default)]
pub struct InMemoryLibraryStore {
    owners: Mutex<HashMap<Owner, OwnerLibrary>>,
}

impl InMemoryLibraryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The result of `operation` on the owner's library, under the store's lock.
    fn with<T>(&self, owner: &Owner, operation: impl FnOnce(&mut OwnerLibrary) -> T) -> T {
        let mut owners = self.owners.lock().unwrap_or_else(PoisonError::into_inner);
        operation(owners.entry(*owner).or_default())
    }
}

#[async_trait]
impl LibraryStore for InMemoryLibraryStore {
    async fn create_project(
        &self,
        owner: &Owner,
        project: ProjectRecord,
    ) -> Result<(), StoreError> {
        self.with(owner, |library| {
            library.projects.insert(project.id, project);
            Ok(())
        })
    }

    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        self.with(owner, |library| library.project(id))
    }

    async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        self.with(owner, |library| {
            let ids: Vec<ProjectId> = library.projects.keys().copied().collect();
            let projects = ids.into_iter().map(|id| library.project(id));
            let projects = projects.collect::<Result<Vec<_>, _>>()?;
            Ok(paginate(projects, &page, project_cursor))
        })
    }

    async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: Name,
        at: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError> {
        self.with(owner, |library| {
            let project = library.projects.get_mut(&id);
            let project = project.ok_or(StoreError::ProjectNotFound)?;
            project.name = name;
            project.updated_at = at;
            library.project(id)
        })
    }

    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError> {
        self.with(owner, |library| {
            library
                .projects
                .remove(&id)
                .ok_or(StoreError::ProjectNotFound)?;
            library
                .animations
                .retain(|_, animation| animation.record.project != id);
            Ok(())
        })
    }

    async fn create_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.with(owner, |library| {
            library.require_project(new.project)?;
            let document_bytes = byte_length(&new.document);
            library.check_quota((0, document_bytes), quota)?;
            let record = AnimationRecord {
                id: new.id,
                project: new.project,
                meta: new.meta,
                document_bytes,
                version: FIRST_VERSION,
                created_at: new.at,
                updated_at: new.at,
            };
            let stored = StoredAnimation {
                record: record.clone(),
                document: new.document,
            };
            library.animations.insert(record.id, stored);
            Ok(record)
        })
    }

    async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, StoreError> {
        self.with(owner, |library| Ok(library.animation(id)?.record.clone()))
    }

    async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        let query = filter.query.map(|query| query.to_lowercase());
        let is_kept = |record: &AnimationRecord| {
            let is_in_project = filter
                .project
                .is_none_or(|project| record.project == project);
            let title = record.meta.title.as_str().to_lowercase();
            is_in_project && query.as_ref().is_none_or(|query| title.contains(query))
        };
        self.with(owner, |library| {
            let records = library.animations.values().map(|stored| &stored.record);
            let records = records.filter(|record| is_kept(record)).cloned().collect();
            Ok(paginate(records, &page, animation_cursor))
        })
    }

    async fn read_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        self.with(owner, |library| {
            let stored = library.animation(id)?;
            Ok((stored.record.clone(), stored.document.clone()))
        })
    }

    async fn write_document(
        &self,
        owner: &Owner,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        self.with(owner, |library| {
            let current = &library.animation(write.id)?.record;
            if current.version != write.expected_version {
                return Err(StoreError::VersionConflict {
                    current: current.version,
                });
            }
            let document_bytes = byte_length(&write.document);
            library.check_quota((current.document_bytes, document_bytes), quota)?;
            let stored = library.animation_mut(write.id)?;
            stored.record.meta = write.meta;
            stored.record.document_bytes = document_bytes;
            stored.record.version += 1;
            stored.record.updated_at = write.at;
            stored.document = write.document;
            Ok(stored.record.clone())
        })
    }

    async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
        at: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError> {
        self.with(owner, |library| {
            library.animation(id)?;
            library.require_project(to)?;
            let stored = library.animation_mut(id)?;
            stored.record.project = to;
            stored.record.updated_at = at;
            Ok(stored.record.clone())
        })
    }

    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError> {
        self.with(owner, |library| {
            let removed = library.animations.remove(&id);
            removed.map(|_| ()).ok_or(StoreError::AnimationNotFound)
        })
    }

    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError> {
        Ok(self.with(owner, |library| library.usage()))
    }

    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError> {
        let mut owners = self.owners.lock().unwrap_or_else(PoisonError::into_inner);
        owners.remove(owner);
        Ok(())
    }
}
