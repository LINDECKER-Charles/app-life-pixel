//! The library folder: where each project and animation lives, and which entries it knows.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::documents::DocumentCache;
use super::files::is_not_found;
use crate::ids::{AnimationId, ProjectId};
use crate::ports::library_store::StoreError;

/// The folder of the projects.
const PROJECTS_FOLDER: &str = "projects";
/// A project's own file, in its folder.
const PROJECT_FILE: &str = "project.json";
/// The folder of a project's animations, in its folder.
const ANIMATIONS_FOLDER: &str = "animations";
/// The extension of a document file.
const DOCUMENT_EXTENSION: &str = "json";

/// A library folder, and the summaries of the documents read from it.
pub(super) struct Folder {
    pub(super) root: PathBuf,
    pub(super) cache: DocumentCache,
}

impl Folder {
    /// The folder of the project `id`.
    pub(super) fn project_folder(&self, id: ProjectId) -> PathBuf {
        self.root.join(PROJECTS_FOLDER).join(id.to_string())
    }

    /// The file of the project `id`.
    pub(super) fn project_file(&self, id: ProjectId) -> PathBuf {
        self.project_folder(id).join(PROJECT_FILE)
    }

    /// The folder of the animations of the project `id`.
    pub(super) fn animations_folder(&self, project: ProjectId) -> PathBuf {
        self.project_folder(project).join(ANIMATIONS_FOLDER)
    }

    /// The document file of the animation `id` of `project`.
    pub(super) fn document_file(&self, project: ProjectId, id: AnimationId) -> PathBuf {
        let name = format!("{id}.{DOCUMENT_EXTENSION}");
        self.animations_folder(project).join(name)
    }

    /// The ids of the project folders; other entries are ignored.
    pub(super) fn project_ids(&self) -> Result<Vec<ProjectId>, StoreError> {
        let names = entry_names(&self.root.join(PROJECTS_FOLDER))?;
        let ids = names.iter().filter_map(|name| name.parse::<Uuid>().ok());
        Ok(ids.map(ProjectId::from_uuid).collect())
    }

    /// The ids of the document files of `project`; other entries are ignored.
    pub(super) fn animation_ids(&self, project: ProjectId) -> Result<Vec<AnimationId>, StoreError> {
        let names = entry_names(&self.animations_folder(project))?;
        let ids = names.iter().filter_map(|name| document_id(name));
        Ok(ids.map(AnimationId::from_uuid).collect())
    }

    /// The project whose folder holds the document of the animation `id`.
    pub(super) fn project_of(&self, id: AnimationId) -> Result<ProjectId, StoreError> {
        let projects = self.project_ids()?.into_iter();
        let mut holding = projects.filter(|project| {
            self.project_file(*project).is_file() && self.document_file(*project, id).is_file()
        });
        holding.next().ok_or(StoreError::AnimationNotFound)
    }
}

/// The storage failure `error`; the service logs its detail.
pub(super) fn unavailable(error: impl ToString) -> StoreError {
    StoreError::Unavailable(error.to_string())
}

/// The names of the entries of `folder`: none when it does not exist.
fn entry_names(folder: &Path) -> Result<Vec<String>, StoreError> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if is_not_found(&error) => return Ok(Vec::new()),
        Err(error) => return Err(unavailable(error)),
    };
    let names = entries.map(|entry| Ok(entry?.file_name()));
    let names: io::Result<Vec<_>> = names.collect();
    let names = names.map_err(unavailable)?.into_iter();
    Ok(names.filter_map(|name| name.into_string().ok()).collect())
}

/// The id of a document file named `name`: `<uuid>.json`.
fn document_id(name: &str) -> Option<Uuid> {
    let stem = name.strip_suffix(DOCUMENT_EXTENSION)?.strip_suffix('.')?;
    stem.parse().ok()
}
