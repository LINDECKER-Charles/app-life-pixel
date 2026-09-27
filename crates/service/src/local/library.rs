//! The local library as a [`LibraryStore`]: each operation runs on tokio's blocking pool.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_core::Name;
use time::OffsetDateTime;

use super::folder::{Folder, unavailable};
use super::opening::{LocalLibraryError, open_manifest};
use crate::ids::{AnimationId, ProjectId};
use crate::owner::Owner;
use crate::paging::{Page, PageRequest};
use crate::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, LibraryStore, NewAnimationRecord,
    ProjectRecord, StoreError,
};

/// The library store of the CLI and the desktop app: projects and animations as files in a
/// library folder, for [`Owner::Local`] only — any other owner sees an empty library and can
/// change nothing. No quota applies unless one is given, and nothing is recorded.
#[derive(Clone)]
pub struct LocalLibrary {
    folder: Arc<Folder>,
}

impl LocalLibrary {
    /// The library in the folder `path`, writing its `library.json` when it has none. Blocks on
    /// the file system.
    ///
    /// # Errors
    ///
    /// [`LocalLibraryError::Unavailable`] when the folder is missing or unwritable, or its
    /// `library.json` is not a library's; [`LocalLibraryError::UnsupportedVersion`] when a later
    /// version of Life Pixel wrote it.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, LocalLibraryError> {
        let root = path.into();
        open_manifest(&root)?;
        let folder = Folder {
            root,
            cache: Default::default(),
        };
        Ok(Self {
            folder: Arc::new(folder),
        })
    }

    /// The library in the folder `path`, created with its parents when missing: the default
    /// library on first launch. Blocks on the file system.
    ///
    /// # Errors
    ///
    /// As [`open`](Self::open), and [`LocalLibraryError::Unavailable`] when the folder cannot
    /// be created.
    pub fn create(path: impl Into<PathBuf>) -> Result<Self, LocalLibraryError> {
        let root = path.into();
        if let Err(error) = fs::create_dir_all(&root) {
            tracing::warn!(path = %root.display(), %error, "library folder not created");
        }
        Self::open(root)
    }

    /// The library folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.folder.root
    }

    /// The result of `work` on the folder, on tokio's blocking pool.
    async fn run<T, F>(&self, work: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Folder) -> Result<T, StoreError> + Send + 'static,
    {
        let folder = Arc::clone(&self.folder);
        let task = tokio::task::spawn_blocking(move || work(&folder));
        task.await.map_err(unavailable)?
    }
}

/// Whether `owner` is the one owner of a local library.
fn is_local(owner: &Owner) -> bool {
    *owner == Owner::Local
}

/// What another owner lists: nothing.
fn empty_page<T>() -> Page<T> {
    Page {
        items: Vec::new(),
        next_cursor: None,
    }
}

#[async_trait]
impl LibraryStore for LocalLibrary {
    async fn create_project(
        &self,
        owner: &Owner,
        project: ProjectRecord,
    ) -> Result<(), StoreError> {
        if !is_local(owner) {
            return Err(unavailable(
                "a local library holds the local owner's projects only",
            ));
        }
        self.run(move |folder| folder.create_project(&project))
            .await
    }

    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::ProjectNotFound);
        }
        self.run(move |folder| folder.project(id)).await
    }

    async fn list_projects(
        &self,
        owner: &Owner,
        page: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        if !is_local(owner) {
            return Ok(empty_page());
        }
        self.run(move |folder| folder.list_projects(&page)).await
    }

    async fn rename_project(
        &self,
        owner: &Owner,
        id: ProjectId,
        name: Name,
        at: OffsetDateTime,
    ) -> Result<ProjectRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::ProjectNotFound);
        }
        self.run(move |folder| folder.rename_project(id, (name, at)))
            .await
    }

    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError> {
        if !is_local(owner) {
            return Err(StoreError::ProjectNotFound);
        }
        self.run(move |folder| folder.delete_project(id)).await
    }

    async fn create_animation(
        &self,
        owner: &Owner,
        new: NewAnimationRecord,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::ProjectNotFound);
        }
        self.run(move |folder| folder.create_animation(new, quota))
            .await
    }

    async fn get_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<AnimationRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::AnimationNotFound);
        }
        self.run(move |folder| folder.animation(id)).await
    }

    async fn list_animations(
        &self,
        owner: &Owner,
        filter: AnimationFilter,
        page: PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        if !is_local(owner) {
            return Ok(empty_page());
        }
        self.run(move |folder| folder.list_animations(&filter, &page))
            .await
    }

    async fn read_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        if !is_local(owner) {
            return Err(StoreError::AnimationNotFound);
        }
        self.run(move |folder| folder.read_document(id)).await
    }

    async fn write_document(
        &self,
        owner: &Owner,
        write: DocumentWrite,
        quota: Option<u64>,
    ) -> Result<AnimationRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::AnimationNotFound);
        }
        self.run(move |folder| folder.write_document(write, quota))
            .await
    }

    async fn move_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        to: ProjectId,
        at: OffsetDateTime,
    ) -> Result<AnimationRecord, StoreError> {
        if !is_local(owner) {
            return Err(StoreError::AnimationNotFound);
        }
        self.run(move |folder| folder.move_animation(id, (to, at)))
            .await
    }

    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError> {
        if !is_local(owner) {
            return Err(StoreError::AnimationNotFound);
        }
        self.run(move |folder| folder.delete_animation(id)).await
    }

    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError> {
        if !is_local(owner) {
            return Ok(0);
        }
        self.run(Folder::usage).await
    }

    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError> {
        if !is_local(owner) {
            return Ok(());
        }
        self.run(Folder::delete_everything).await
    }
}
