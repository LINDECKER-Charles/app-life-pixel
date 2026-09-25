//! The library's storage: one port, because its operations are atomic across the index and the
//! documents — a write checks the version and the quota and updates the usage as one change.
//!
//! Every adapter passes the contract suite `testing::library_store_contract!`, feature
//! `testing`. Its rules:
//!
//! - An owner never sees another owner's projects or animations: they answer not found.
//! - Lists run from the most recently updated, ties broken by the larger id; see
//!   [`paging`](crate::paging).
//! - A document is stored and read back byte for byte; `document_bytes` is its length.
//! - A write with the current version gives a new version, different from the one it replaced,
//!   below 2^53; with another version, [`StoreError::VersionConflict`].
//! - A create or a write passes the quota as [`StorageChange::check`] says; `quota: None` means no
//!   quota. Deleting a project deletes its animations and frees their usage.
//!
//! [`StorageChange::check`]: crate::quota::StorageChange::check

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use thiserror::Error;
use time::OffsetDateTime;

use crate::ids::{AnimationId, ProjectId};
use crate::owner::Owner;
use crate::paging::{Page, PageRequest};

/// Where projects, animations and their documents are kept.
#[async_trait]
#[allow(clippy::too_many_arguments)] // The port's signatures are the contract of service.md.
pub trait LibraryStore: Send + Sync {
    /// Adds `project`.
    async fn create_project(&self, owner: &Owner, project: ProjectRecord)
    -> Result<(), StoreError>;
    /// The project `id`.
    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError>;
    /// A page of the owner's projects.
    async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError>;
    /// Renames the project `id`, updated `at`.
    async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: Name,
        at: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError>;
    /// Deletes the project `id` and its animations.
    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError>;
    /// Adds an animation to an existing project, within `quota` bytes of usage.
    async fn create_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError>;
    /// The animation `id`, without its document.
    async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, StoreError>;
    /// A page of the owner's animations that `filter` keeps.
    async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError>;
    /// The animation `id` and its document.
    async fn read_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError>;
    /// Replaces a document when its version is the expected one, within `quota` bytes of usage.
    async fn write_document(
        &self,
        owner: &Owner,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError>;
    /// Moves the animation `id` to the project `to`, updated `at`.
    async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
        at: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError>;
    /// Deletes the animation `id`.
    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError>;
    /// The bytes of the owner's documents.
    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError>;
    /// Deletes every project and animation of the owner.
    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError>;
}

/// A project as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRecord {
    /// Its id.
    pub id: ProjectId,
    /// Its name.
    pub name: Name,
    /// How many animations it holds.
    pub animation_count: u32,
    /// When it was created.
    pub created_at: OffsetDateTime,
    /// When it was last renamed, or created.
    pub updated_at: OffsetDateTime,
}

/// What lists show of an animation, read from its document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationMeta {
    /// The document's title.
    pub title: Name,
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// How many frames it has.
    pub frame_count: u16,
}

/// A new animation and its document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAnimationRecord {
    /// Its id.
    pub id: AnimationId,
    /// The project it goes into.
    pub project: ProjectId,
    /// What its document says.
    pub meta: AnimationMeta,
    /// Its document, serialized by `core`.
    pub document: Bytes,
    /// When it is created.
    pub at: OffsetDateTime,
}

/// An animation as stored, without its document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationRecord {
    /// Its id.
    pub id: AnimationId,
    /// The project it belongs to.
    pub project: ProjectId,
    /// What its document says.
    pub meta: AnimationMeta,
    /// The length of its document.
    pub document_bytes: u64,
    /// Its document's version: what a write must expect.
    pub version: u64,
    /// When it was created.
    pub created_at: OffsetDateTime,
    /// When it was last written or moved.
    pub updated_at: OffsetDateTime,
}

/// A new document for an existing animation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentWrite {
    /// The animation.
    pub id: AnimationId,
    /// The version the writer read: the write fails when it is no longer current.
    pub expected_version: u64,
    /// What the new document says.
    pub meta: AnimationMeta,
    /// The new document, serialized by `core`.
    pub document: Bytes,
    /// When it is written.
    pub at: OffsetDateTime,
}

/// Which animations a list keeps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimationFilter {
    /// Only this project's.
    pub project: Option<ProjectId>,
    /// Only those whose title holds this text, case-insensitively.
    pub query: Option<String>,
}

/// Why the store refused an operation.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StoreError {
    /// No such project for this owner.
    #[error("project not found")]
    ProjectNotFound,
    /// No such animation for this owner.
    #[error("animation not found")]
    AnimationNotFound,
    /// The document's version is not the expected one.
    #[error("version conflict, current version {current}")]
    VersionConflict {
        /// The document's current version.
        current: u64,
    },
    /// The change would take the owner's usage beyond the quota.
    #[error("storage quota exceeded: {used} + {requested} > {limit} bytes")]
    QuotaExceeded {
        /// The owner's usage, in bytes.
        used: u64,
        /// The quota, in bytes.
        limit: u64,
        /// The bytes the change would add to the usage.
        requested: u64,
    },
    /// The storage failed; the detail is logged, never shown.
    #[error("storage unavailable: {0}")]
    Unavailable(String),
}
