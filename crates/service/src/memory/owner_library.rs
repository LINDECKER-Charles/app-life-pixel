//! One owner's projects and animations, in memory.

use std::collections::HashMap;

use bytes::Bytes;

use crate::ids::{AnimationId, ProjectId};
use crate::paging::Cursor;
use crate::ports::library_store::{AnimationRecord, ProjectRecord, StoreError};
use crate::quota::StorageChange;

/// An animation and its document.
pub(super) struct StoredAnimation {
    pub(super) record: AnimationRecord,
    pub(super) document: Bytes,
}

/// An owner's library.
#[derive(Default)]
pub(super) struct OwnerLibrary {
    /// The projects, their `animation_count` left at 0: it is counted when read.
    pub(super) projects: HashMap<ProjectId, ProjectRecord>,
    pub(super) animations: HashMap<AnimationId, StoredAnimation>,
}

impl OwnerLibrary {
    /// The project `id`, with its animations counted.
    pub(super) fn project(&self, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        let project = self.projects.get(&id).ok_or(StoreError::ProjectNotFound)?;
        let count = self
            .animations
            .values()
            .filter(|animation| animation.record.project == id)
            .count();
        Ok(ProjectRecord {
            animation_count: u32::try_from(count).unwrap_or(u32::MAX),
            ..project.clone()
        })
    }

    /// Fails unless the project `id` exists.
    pub(super) fn require_project(&self, id: ProjectId) -> Result<(), StoreError> {
        self.projects
            .contains_key(&id)
            .then_some(())
            .ok_or(StoreError::ProjectNotFound)
    }

    pub(super) fn animation(&self, id: AnimationId) -> Result<&StoredAnimation, StoreError> {
        self.animations
            .get(&id)
            .ok_or(StoreError::AnimationNotFound)
    }

    pub(super) fn animation_mut(
        &mut self,
        id: AnimationId,
    ) -> Result<&mut StoredAnimation, StoreError> {
        self.animations
            .get_mut(&id)
            .ok_or(StoreError::AnimationNotFound)
    }

    /// The bytes of every document.
    pub(super) fn usage(&self) -> u64 {
        self.animations
            .values()
            .map(|animation| animation.record.document_bytes)
            .sum()
    }

    /// Checks replacing `freed` bytes with `added` against `quota`.
    pub(super) fn check_quota(
        &self,
        (freed, added): (u64, u64),
        quota: Option<u64>,
    ) -> Result<(), StoreError> {
        let used = self.usage();
        StorageChange { used, freed, added }.check(quota)
    }
}

/// Where a project stands in a list.
pub(super) fn project_cursor(project: &ProjectRecord) -> Cursor {
    Cursor {
        updated_at: project.updated_at,
        id: project.id.uuid(),
    }
}

/// Where an animation stands in a list.
pub(super) fn animation_cursor(animation: &AnimationRecord) -> Cursor {
    Cursor {
        updated_at: animation.updated_at,
        id: animation.id.uuid(),
    }
}

/// The length of `document`, as stored.
pub(super) fn byte_length(document: &Bytes) -> u64 {
    u64::try_from(document.len()).unwrap_or(u64::MAX)
}
