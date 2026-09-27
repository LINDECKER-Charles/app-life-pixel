//! Animations: create, import, read, list, move, delete.

use bytes::Bytes;
use life_pixel_core::NewAnimation;

use super::parsing::{self, Document};
use super::{ANIMATION_CREATED, Library, LibraryError};
use crate::ids::{AnimationId, ProjectId};
use crate::owner::Owner;
use crate::paging::{Page, PageRequest};
use crate::ports::library_store::{AnimationFilter, AnimationRecord, NewAnimationRecord};

impl Library {
    /// Creates a blank animation in `project`, as `core::Animation::new` makes it: for MCP.
    ///
    /// # Errors
    ///
    /// The `document.*` code of an invalid spec; `library.project_not_found`;
    /// `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn create_animation(
        &self,
        owner: &Owner,
        project: ProjectId,
        spec: NewAnimation,
    ) -> Result<AnimationRecord, LibraryError> {
        let document = parsing::blank(spec).await?;
        let new = self.new_animation(project, document);
        self.insert_animation(owner, new).await
    }

    /// Creates an animation in `project` from a serialized document: the app's first save, the
    /// desktop's import.
    ///
    /// # Errors
    ///
    /// The `document.*` code of an invalid document; `library.project_not_found`;
    /// `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn import_animation(
        &self,
        owner: &Owner,
        project: ProjectId,
        document: Bytes,
    ) -> Result<AnimationRecord, LibraryError> {
        let document = parsing::parse(document).await?;
        let new = self.new_animation(project, document);
        self.insert_animation(owner, new).await
    }

    /// The animation `id`, without its document.
    ///
    /// # Errors
    ///
    /// `library.animation_not_found`; `service.unavailable`.
    pub async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, LibraryError> {
        let animation = self.store().get_animation(owner, id).await;
        animation.map_err(|error| self.refused(owner, error))
    }

    /// A page of the owner's animations, from the most recently updated: those of a project,
    /// those whose title holds a text, or all.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, LibraryError> {
        let animations = self.store().list_animations(owner, filter, page).await;
        animations.map_err(|error| self.refused(owner, error))
    }

    /// Moves the animation `id` to the project `to`.
    ///
    /// # Errors
    ///
    /// `library.animation_not_found`; `library.project_not_found`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
    ) -> Result<AnimationRecord, LibraryError> {
        let at = self.ports.clock.now();
        let moved = self.store().move_animation(owner, id, to, at).await;
        moved.map_err(|error| self.refused(owner, error))
    }

    /// Deletes the animation `id`. Deleting always works, even above the quota.
    ///
    /// # Errors
    ///
    /// `library.animation_not_found`; `service.unavailable`.
    pub async fn delete_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(), LibraryError> {
        let deleted = self.store().delete_animation(owner, id).await;
        deleted.map_err(|error| self.refused(owner, error))
    }

    /// The record of `document` as a new animation of `project`, with a new id.
    pub(super) fn new_animation(
        &self,
        project: ProjectId,
        document: Document,
    ) -> NewAnimationRecord {
        NewAnimationRecord {
            id: AnimationId::from_uuid(self.ports.ids.new_id()),
            project,
            meta: document.meta,
            document: document.bytes,
            at: self.ports.clock.now(),
        }
    }

    /// Stores `new` under the owner's quota.
    pub(super) async fn insert_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
    ) -> Result<AnimationRecord, LibraryError> {
        let quota = self.quota(owner);
        let created = self.store().create_animation(owner, new, quota).await;
        let created = created.map_err(|error| self.refused(owner, error))?;
        self.record(owner, ANIMATION_CREATED);
        Ok(created)
    }
}
