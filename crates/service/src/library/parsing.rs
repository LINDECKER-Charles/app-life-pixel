//! Documents through `core`, on tokio's blocking pool: parsing and validating never runs on the
//! executor.

use bytes::Bytes;
use life_pixel_core::limits::MAX_DOCUMENT_BYTES;
use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{Animation, DocumentError, Name, NewAnimation};
use serde_json::Value;

use super::LibraryError;
use crate::ports::library_store::AnimationMeta;

/// The document field holding the title.
const TITLE_FIELD: &str = "title";

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

/// The document `bytes` with the title `title`, checked again by `core`. The title is set in the
/// current document version's JSON until `core` offers a title edit.
fn retitled(bytes: &[u8], title: &Name) -> Result<Document, DocumentError> {
    let current = write_document(&read_document(bytes)?)?;
    let mut document: Value =
        serde_json::from_str(&current).map_err(|_| DocumentError::Malformed)?;
    let fields = document.as_object_mut().ok_or(DocumentError::Malformed)?;
    fields.insert(TITLE_FIELD.to_owned(), title.as_str().into());
    let bytes = serde_json::to_vec(&document).map_err(|_| DocumentError::Malformed)?;
    serialize(&read_document(&bytes)?)
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
