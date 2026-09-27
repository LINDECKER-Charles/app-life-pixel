//! Documents through `core`, on tokio's blocking pool: parsing and validating never runs on the
//! executor.

use bytes::Bytes;
use life_pixel_core::edit::{self, EditError, Operation};
use life_pixel_core::limits::MAX_DOCUMENT_BYTES;
use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{Animation, DocumentError, Name, NewAnimation};

use super::LibraryError;
use crate::ports::library_store::AnimationMeta;

/// A valid document as `core` serializes it, and what lists show of it.
pub(super) struct Document {
    pub(super) meta: AnimationMeta,
    pub(super) bytes: Bytes,
}

/// The document of `bytes`: size against `MAX_DOCUMENT_BYTES` first, then parsed, validated and
/// serialized again by `core`.
pub(super) async fn parse(bytes: Bytes) -> Result<Document, LibraryError> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(DocumentError::TooLarge.into());
    }
    blocking(move || serialize(&read_document(&bytes)?)).await
}

/// The document of a blank animation.
pub(super) async fn blank(spec: NewAnimation) -> Result<Document, LibraryError> {
    blocking(move || serialize(&Animation::new(spec)?)).await
}

/// The document of `bytes`, titled `title`.
pub(super) async fn retitle(bytes: Bytes, title: Name) -> Result<Document, LibraryError> {
    blocking(move || retitled(&bytes, &title)).await
}

/// The document `bytes` with the title `title`, set through `core`'s `Operation::SetTitle` and
/// checked again by the model's rules.
fn retitled(bytes: &[u8], title: &Name) -> Result<Document, DocumentError> {
    let mut animation = read_document(bytes)?;
    let operation = Operation::SetTitle {
        title: title.as_str().to_owned(),
    };
    edit::apply(&mut animation, &operation).map_err(|error| match error {
        EditError::Document(document) => document,
        _ => DocumentError::Malformed,
    })?;
    serialize(&animation)
}

fn serialize(animation: &Animation) -> Result<Document, DocumentError> {
    let text = write_document(animation)?;
    let meta = AnimationMeta {
        title: animation.title().clone(),
        width: animation.width(),
        height: animation.height(),
        frame_count: u16::try_from(animation.frames().len()).unwrap_or(u16::MAX),
    };
    Ok(Document {
        meta,
        bytes: Bytes::from(text),
    })
}

async fn blocking<T, F>(work: F) -> Result<T, LibraryError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, DocumentError> + Send + 'static,
{
    let outcome = tokio::task::spawn_blocking(work).await.map_err(|error| {
        tracing::error!(%error, "document task failed");
        LibraryError::Unavailable
    })?;
    Ok(outcome?)
}
