//! Projects: create, read, list, rename, delete.

use life_pixel_core::Name;

use super::{Library, LibraryError};
use crate::ids::ProjectId;
use crate::owner::Owner;
use crate::paging::{Page, PageRequest};
use crate::ports::library_store::ProjectRecord;

impl Library {
    /// Creates an empty project named `name`.
    ///
    /// # Errors
    ///
    /// `document.name` for an invalid name; `service.unavailable`.
    pub async fn create_project(
        &self,
        owner: &Owner,
        name: &str,
    ) -> Result<ProjectRecord, LibraryError> {
        let project = self.new_project(Name::new(name)?);
        let created = project.clone();
        self.store()
            .create_project(owner, project)
            .await
            .map_err(|error| self.refused(owner, error))?;
        Ok(created)
    }

    /// The project `id`.
    ///
    /// # Errors
    ///
    /// `library.project_not_found`; `service.unavailable`.
    pub async fn get_project(
        &self,
        owner: &Owner,
        id: ProjectId,
    ) -> Result<ProjectRecord, LibraryError> {
        let project = self.store().get_project(owner, id).await;
        project.map_err(|error| self.refused(owner, error))
    }

    /// A page of the owner's projects, from the most recently updated.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, LibraryError> {
        let projects = self.store().list_projects(owner, page).await;
        projects.map_err(|error| self.refused(owner, error))
    }

    /// Renames the project `id`.
    ///
    /// # Errors
    ///
    /// `document.name` for an invalid name; `library.project_not_found`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: &str,
    ) -> Result<ProjectRecord, LibraryError> {
        let name = Name::new(name)?;
        let at = self.ports.clock.now();
        let project = self.store().rename_project(owner, id, name, at).await;
        project.map_err(|error| self.refused(owner, error))
    }

    /// Deletes the project `id` and its animations.
    ///
    /// # Errors
    ///
    /// `library.project_not_found`; `service.unavailable`.
    pub async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), LibraryError> {
        let deleted = self.store().delete_project(owner, id).await;
        deleted.map_err(|error| self.refused(owner, error))
    }

    /// A new, empty project record named `name`.
    pub(super) fn new_project(&self, name: Name) -> ProjectRecord {
        let now = self.ports.clock.now();
        ProjectRecord {
            id: ProjectId::from_uuid(self.ports.ids.new_id()),
            name,
            animation_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}
