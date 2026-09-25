//! Changing animations, under the library's lock: a write checks the version of the file on
//! disk, so a change made behind the library's back is a conflict, never lost.

use std::fs::{self, File, Metadata};
use std::io::Read as _;
use std::path::Path;
use std::time::SystemTime;

use time::OffsetDateTime;

use super::documents::{Summary, version_of};
use super::files::{is_not_found, locked, touch, write_atomically};
use super::folder::{Folder, unavailable};
use crate::ids::{AnimationId, ProjectId};
use crate::ports::library_store::{
    AnimationMeta, AnimationRecord, DocumentWrite, NewAnimationRecord, StoreError,
};
use crate::quota::StorageChange;

/// A document to write, and where it belongs.
struct DocumentFile<'a> {
    project: ProjectId,
    id: AnimationId,
    document: &'a [u8],
    meta: AnimationMeta,
    /// The creation time of the file it replaces, kept where the platform allows.
    created: Option<SystemTime>,
}

impl Folder {
    /// Adds an animation to an existing project, within `quota` bytes of usage.
    pub(super) fn create_animation(
        &self,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        locked(&self.root, || {
            self.stored_project(new.project)?;
            self.check_quota((0, byte_length(&new.document)), quota)?;
            let folder = self.animations_folder(new.project);
            fs::create_dir_all(folder).map_err(unavailable)?;
            self.store(DocumentFile {
                project: new.project,
                id: new.id,
                document: &new.document,
                meta: new.meta,
                created: None,
            })
        })
        .map_err(unavailable)?
    }

    /// Replaces a document when the version of its file is the expected one, within `quota`.
    pub(super) fn write_document(
        &self,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        locked(&self.root, || {
            let project = self.project_of(write.id)?;
            let path = self.document_file(project, write.id);
            let (metadata, current) = read_current(&path)?;
            let version = version_of(&current);
            if version != write.expected_version {
                return Err(StoreError::VersionConflict { current: version });
            }
            let change = (byte_length(&current), byte_length(&write.document));
            self.check_quota(change, quota)?;
            self.store(DocumentFile {
                project,
                id: write.id,
                document: &write.document,
                meta: write.meta,
                created: metadata.created().ok(),
            })
        })
        .map_err(unavailable)?
    }

    /// Moves the animation `id` to the project `to`, updated `at`.
    pub(super) fn move_animation(
        &self,
        id: AnimationId,
        (to, at): (ProjectId, OffsetDateTime),
    ) -> Result<AnimationRecord, StoreError> {
        locked(&self.root, || {
            let from = self.project_of(id)?;
            self.stored_project(to)?;
            let target = self.document_file(to, id);
            fs::create_dir_all(self.animations_folder(to)).map_err(unavailable)?;
            fs::rename(self.document_file(from, id), &target).map_err(unavailable)?;
            touch(&target, SystemTime::from(at)).map_err(unavailable)?;
            self.record(to, id)?.ok_or(StoreError::AnimationNotFound)
        })
        .map_err(unavailable)?
    }

    /// Deletes the animation `id`, whether its document parses or not.
    pub(super) fn delete_animation(&self, id: AnimationId) -> Result<(), StoreError> {
        locked(&self.root, || {
            let path = self.document_file(self.project_of(id)?, id);
            fs::remove_file(&path).map_err(unavailable)?;
            self.cache.forget(&path);
            Ok(())
        })
        .map_err(unavailable)?
    }

    /// Writes the document `file` describes, and gives its record.
    fn store(&self, file: DocumentFile<'_>) -> Result<AnimationRecord, StoreError> {
        let path = self.document_file(file.project, file.id);
        let metadata = write_atomically(&path, file.document, file.created);
        let metadata = metadata.map_err(unavailable)?;
        let version = version_of(file.document);
        let summary = Summary {
            meta: file.meta,
            version,
        };
        self.cache.insert(&path, (&metadata, summary.clone()));
        let modified = metadata.modified().map_err(unavailable)?;
        let created = metadata.created().unwrap_or(modified);
        Ok(AnimationRecord {
            id: file.id,
            project: file.project,
            meta: summary.meta,
            document_bytes: metadata.len(),
            version,
            created_at: OffsetDateTime::from(created),
            updated_at: OffsetDateTime::from(modified),
        })
    }

    /// Checks replacing `freed` bytes with `added` against `quota`; the usage is only counted
    /// when there is a quota.
    fn check_quota(
        &self,
        (freed, added): (u64, u64),
        quota: Option<u64>,
    ) -> Result<(), StoreError> {
        if quota.is_none() {
            return Ok(());
        }
        let used = self.usage()?;
        StorageChange { used, freed, added }.check(quota)
    }
}

/// The metadata and bytes of the document file at `path`, from one handle.
fn read_current(path: &Path) -> Result<(Metadata, Vec<u8>), StoreError> {
    let read = File::open(path).and_then(|mut file| {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok((file.metadata()?, bytes))
    });
    read.map_err(|error| {
        if is_not_found(&error) {
            StoreError::AnimationNotFound
        } else {
            unavailable(error)
        }
    })
}

/// The length of `bytes`, as stored.
fn byte_length(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}
