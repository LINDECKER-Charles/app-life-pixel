//! An owner's library copied into a local library in a temporary folder: the local adapter lays
//! it out, so that the export is what the desktop app opens. Documents are read one at a time.

use life_pixel_core::limits::PAGE_SIZE_MAX;
use tempfile::TempDir;

use super::blocking;
use crate::accounts::AccountsError;
use crate::ids::AnimationId;
use crate::library::{Library, LibraryError};
use crate::local::LocalLibrary;
use crate::owner::Owner;
use crate::paging::PageRequest;
use crate::ports::LibraryStore;
use crate::ports::library_store::{AnimationFilter, NewAnimationRecord, StoreError};

/// The library of `owner` in `library`, copied into a local library in a new temporary folder,
/// removed when dropped. An animation deleted during the copy, or created in a project created
/// during it, is left out.
pub(super) async fn stage(library: &Library, owner: &Owner) -> Result<TempDir, AccountsError> {
    let (folder, staged) = blocking(create).await??;
    let copy = LibraryCopy {
        library,
        owner,
        staged: &staged,
    };
    copy.projects().await?;
    copy.animations().await?;
    Ok(folder)
}

/// A new temporary folder, holding an empty local library.
fn create() -> Result<(TempDir, LocalLibrary), AccountsError> {
    let folder = tempfile::tempdir().map_err(unavailable)?;
    let staged = LocalLibrary::open(folder.path()).map_err(unavailable)?;
    Ok((folder, staged))
}

/// A copy of the library of `owner` into `staged`.
struct LibraryCopy<'a> {
    library: &'a Library,
    owner: &'a Owner,
    staged: &'a LocalLibrary,
}

impl LibraryCopy<'_> {
    /// Copies every project, with its id, name and dates.
    async fn projects(&self) -> Result<(), AccountsError> {
        let mut request = first_page();
        loop {
            let page = self
                .library
                .list_projects(self.owner, request.clone())
                .await;
            let page = page.map_err(unavailable)?;
            for project in page.items {
                let created = self.staged.create_project(&Owner::Local, project).await;
                created.map_err(unavailable)?;
            }
            let Some(next) = page.next_cursor else {
                return Ok(());
            };
            request.cursor = Some(next);
        }
    }

    /// Copies every animation with its document, into the project of the same id.
    async fn animations(&self) -> Result<(), AccountsError> {
        let mut request = first_page();
        loop {
            let filter = AnimationFilter::default();
            let page = self
                .library
                .list_animations(self.owner, filter, request.clone());
            let page = page.await.map_err(unavailable)?;
            for animation in page.items {
                self.animation(animation.id).await?;
            }
            let Some(next) = page.next_cursor else {
                return Ok(());
            };
            request.cursor = Some(next);
        }
    }

    /// Copies the animation `id` and its document, unless it or its project is gone.
    async fn animation(&self, id: AnimationId) -> Result<(), AccountsError> {
        let (record, document) = match self.library.open_document(self.owner, id).await {
            Ok(opened) => opened,
            Err(LibraryError::AnimationNotFound) => return Ok(()),
            Err(error) => return Err(unavailable(error)),
        };
        let copy = NewAnimationRecord {
            id: record.id,
            project: record.project,
            meta: record.meta,
            document,
            at: record.updated_at,
        };
        match self
            .staged
            .create_animation(&Owner::Local, copy, None)
            .await
        {
            Ok(_) | Err(StoreError::ProjectNotFound) => Ok(()),
            Err(error) => Err(unavailable(error)),
        }
    }
}

/// The first page of a list, as long as a page may be.
fn first_page() -> PageRequest {
    PageRequest::new(None, u16::try_from(PAGE_SIZE_MAX).ok())
}

/// `service.unavailable`, logging `error`.
fn unavailable(error: impl std::fmt::Display) -> AccountsError {
    tracing::error!(%error, "a data export could not be staged");
    AccountsError::Unavailable
}
