//! Duplicates: a copy gets new ids, and a project's copy is checked against the quota whole.

use life_pixel_core::Name;
use life_pixel_core::limits::PAGE_SIZE_MAX;

use super::parsing::{self, Document};
use super::{Library, LibraryError};
use crate::ids::{AnimationId, ProjectId};
use crate::owner::Owner;
use crate::paging::PageRequest;
use crate::ports::library_store::{AnimationFilter, AnimationRecord, ProjectRecord};
use crate::quota::StorageChange;

impl Library {
    /// Copies the project `id` and its animations into a new project named `name`. The whole
    /// project's size is checked against the quota first, then it is copied.
    ///
    /// # Errors
    ///
    /// `document.name` for an invalid name; `library.project_not_found`;
    /// `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn duplicate_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: &str,
    ) -> Result<ProjectRecord, LibraryError> {
        let copy = self.new_project(Name::new(name)?);
        self.get_project(owner, id).await?;
        let animations = self.project_animations(owner, id).await?;
        self.check_copy_quota(owner, &animations).await?;
        let created = self.store().create_project(owner, copy.clone()).await;
        created.map_err(|error| self.refused(owner, error))?;
        for animation in animations {
            let (_, bytes) = self.open_document(owner, animation.id).await?;
            let document = Document {
                meta: animation.meta,
                bytes,
            };
            let new = self.new_animation(copy.id, document);
            self.insert_animation(owner, new).await?;
        }
        self.get_project(owner, copy.id).await
    }

    /// Copies the animation `id` with the title `title`, into `project`, or beside the original
    /// when `None`.
    ///
    /// # Errors
    ///
    /// `document.name` for an invalid title; `library.animation_not_found`;
    /// `library.project_not_found`; `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn duplicate_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        title: &str,
        project: Option<ProjectId>,
    ) -> Result<AnimationRecord, LibraryError> {
        let title = Name::new(title)?;
        let (original, bytes) = self.open_document(owner, id).await?;
        let document = parsing::retitle(bytes, title).await?;
        let new = self.new_animation(project.unwrap_or(original.project), document);
        self.insert_animation(owner, new).await
    }

    /// Every animation of `project`.
    async fn project_animations(
        &self,
        owner: &Owner,
        project: ProjectId,
    ) -> Result<Vec<AnimationRecord>, LibraryError> {
        let limit = u16::try_from(PAGE_SIZE_MAX).ok();
        let mut request = PageRequest::new(None, limit);
        let mut animations = Vec::new();
        loop {
            let filter = AnimationFilter {
                project: Some(project),
                query: None,
            };
            let page = self.list_animations(owner, filter, request.clone()).await?;
            animations.extend(page.items);
            let Some(next) = page.next_cursor else {
                return Ok(animations);
            };
            request.cursor = Some(next);
        }
    }

    /// Refuses a copy of `animations` that would take the owner beyond the quota.
    async fn check_copy_quota(
        &self,
        owner: &Owner,
        animations: &[AnimationRecord],
    ) -> Result<(), LibraryError> {
        let used = self.store().usage(owner).await;
        let used = used.map_err(|error| self.refused(owner, error))?;
        let change = StorageChange {
            used,
            freed: 0,
            added: animations
                .iter()
                .map(|animation| animation.document_bytes)
                .sum(),
        };
        change
            .check(self.quota(owner))
            .map_err(|error| self.refused(owner, error))
    }
}
