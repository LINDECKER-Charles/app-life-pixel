//! Reading and changing a document: `core` works on the blocking pool, the library stores the
//! result with the version that was read, and a conflict is retried once.

use std::sync::Arc;

use bytes::Bytes;
use life_pixel_core::Animation;
use life_pixel_core::serialize::{read_document, write_document};

use super::{AnimationEditing, AnimationView, EditingError};
use crate::ids::AnimationId;
use crate::library::LibraryError;
use crate::owner::Owner;
use crate::ports::library_store::AnimationRecord;

/// What a change does to an animation, run on the blocking pool — twice on a conflict.
pub(super) type Edit = Arc<dyn Fn(&mut Animation) -> Result<(), EditingError> + Send + Sync>;

impl AnimationEditing {
    /// Applies `edit` to the animation `id` and saves it; on a version conflict, reads the
    /// animation again and retries once.
    pub(super) async fn change(
        &self,
        owner: &Owner,
        (id, edit): (AnimationId, Edit),
    ) -> Result<AnimationView, EditingError> {
        match self.change_once(owner, (id, Arc::clone(&edit))).await {
            Err(EditingError::Library(LibraryError::VersionConflict { .. })) => {
                self.change_once(owner, (id, edit)).await
            }
            outcome => outcome,
        }
    }

    /// The animation `id`, read and validated by `core`.
    pub(super) async fn read(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Animation), EditingError> {
        let (record, bytes) = self.library.open_document(owner, id).await?;
        let animation = blocking(move || Ok(read_document(&bytes)?)).await?;
        Ok((record, animation))
    }

    /// Reads, edits and saves with the version read.
    async fn change_once(
        &self,
        owner: &Owner,
        (id, edit): (AnimationId, Edit),
    ) -> Result<AnimationView, EditingError> {
        let (record, bytes) = self.library.open_document(owner, id).await?;
        let (animation, document) = blocking(move || edited(&bytes, edit.as_ref())).await?;
        let save = self
            .library
            .save_document(owner, id, record.version, document);
        let saved = save.await?;
        Ok(AnimationView::of(&saved, &animation))
    }
}

/// Runs `work` on tokio's blocking pool: `core` and `compiler` never run on the executor.
pub(super) async fn blocking<T, F>(work: F) -> Result<T, EditingError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, EditingError> + Send + 'static,
{
    tokio::task::spawn_blocking(work).await.map_err(|error| {
        tracing::error!(%error, "animation task failed");
        LibraryError::Unavailable
    })?
}

/// The animation of `bytes` after `edit`, and its new document.
fn edited(
    bytes: &[u8],
    edit: &(dyn Fn(&mut Animation) -> Result<(), EditingError> + Send + Sync),
) -> Result<(Animation, Bytes), EditingError> {
    let mut animation = read_document(bytes)?;
    edit(&mut animation)?;
    let document = write_document(&animation)?;
    Ok((animation, Bytes::from(document)))
}
