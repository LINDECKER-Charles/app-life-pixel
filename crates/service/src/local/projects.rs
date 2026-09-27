//! Projects: one folder each, named by its id, with its `project.json`.

use std::fs;

use life_pixel_core::Name;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::files::{is_not_found, locked, write_atomically};
use super::folder::{Folder, unavailable};
use crate::ids::ProjectId;
use crate::paging::{Cursor, Page, PageRequest, paginate};
use crate::ports::library_store::{ProjectRecord, StoreError};

/// What `project.json` holds.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    id: Uuid,
    name: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

impl Folder {
    /// Adds `project`: its folder, its animations' folder and its file.
    pub(super) fn create_project(&self, project: &ProjectRecord) -> Result<(), StoreError> {
        let created = locked(&self.root, || {
            fs::create_dir_all(self.animations_folder(project.id))?;
            self.write_project(project)
        });
        created.and_then(|written| written).map_err(unavailable)
    }

    /// The project `id`, with its animations counted.
    pub(super) fn project(&self, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        let project = self.stored_project(id)?;
        let count = self.records_of(id)?.len();
        Ok(ProjectRecord {
            animation_count: u32::try_from(count).unwrap_or(u32::MAX),
            ..project
        })
    }

    /// A page of the projects.
    pub(super) fn list_projects(
        &self,
        page: &PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        let mut projects = Vec::new();
        for id in self.project_ids()? {
            match self.project(id) {
                Err(StoreError::ProjectNotFound) => {}
                project => projects.push(project?),
            }
        }
        Ok(paginate(projects, page, project_cursor))
    }

    /// Renames the project `id` to `name`, updated `at`.
    pub(super) fn rename_project(
        &self,
        id: ProjectId,
        (name, at): (Name, OffsetDateTime),
    ) -> Result<ProjectRecord, StoreError> {
        locked(&self.root, || {
            let project = self.stored_project(id)?;
            let renamed = ProjectRecord {
                name,
                updated_at: at,
                ..project
            };
            self.write_project(&renamed).map_err(unavailable)
        })
        .map_err(unavailable)??;
        self.project(id)
    }

    /// Deletes the project `id` and its animations.
    pub(super) fn delete_project(&self, id: ProjectId) -> Result<(), StoreError> {
        locked(&self.root, || {
            self.stored_project(id)?;
            self.remove_project(id)
        })
        .map_err(unavailable)?
    }

    /// Deletes every project.
    pub(super) fn delete_everything(&self) -> Result<(), StoreError> {
        locked(&self.root, || {
            let ids = self.project_ids()?;
            ids.into_iter().try_for_each(|id| self.remove_project(id))
        })
        .map_err(unavailable)?
    }

    /// The project `id` as its file says, its animations not counted. A file that cannot be read
    /// or does not parse leaves the project out, with a warning.
    pub(super) fn stored_project(&self, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        let path = self.project_file(id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if is_not_found(&error) => return Err(StoreError::ProjectNotFound),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "project left out: unreadable");
                return Err(StoreError::ProjectNotFound);
            }
        };
        let record = parse_project(&bytes).filter(|project| project.id == id);
        record.ok_or_else(|| {
            tracing::warn!(path = %path.display(), "project left out: malformed");
            StoreError::ProjectNotFound
        })
    }

    /// Writes the file of `project`.
    fn write_project(&self, project: &ProjectRecord) -> std::io::Result<()> {
        let file = ProjectFile {
            id: project.id.uuid(),
            name: project.name.as_str().to_owned(),
            created_at: project.created_at,
            updated_at: project.updated_at,
        };
        let mut text = serde_json::to_vec_pretty(&file)?;
        text.push(b'\n');
        write_atomically(&self.project_file(project.id), &text, None).map(drop)
    }

    /// Removes the folder of the project `id`.
    fn remove_project(&self, id: ProjectId) -> Result<(), StoreError> {
        let folder = self.project_folder(id);
        fs::remove_dir_all(&folder).map_err(unavailable)?;
        self.cache.forget(&folder);
        Ok(())
    }
}

/// The project a `project.json` describes, if it is one.
fn parse_project(bytes: &[u8]) -> Option<ProjectRecord> {
    let file: ProjectFile = serde_json::from_slice(bytes).ok()?;
    Some(ProjectRecord {
        id: ProjectId::from_uuid(file.id),
        name: Name::new(&file.name).ok()?,
        animation_count: 0,
        created_at: file.created_at,
        updated_at: file.updated_at,
    })
}

/// Where a project stands in a list.
fn project_cursor(project: &ProjectRecord) -> Cursor {
    Cursor {
        updated_at: project.updated_at,
        id: project.id.uuid(),
    }
}
