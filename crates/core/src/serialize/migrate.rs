//! Reading a document of any version this build knows: `format` and `version` first, then the
//! parser of that version, followed by the steps up to the current one. A future version 2 adds a
//! `v1 → v2` step here, and every version stays readable.

use serde::Deserialize;

use super::json::{self, DOCUMENT_FORMAT, DOCUMENT_VERSION};
use crate::error::DocumentError;
use crate::limits::MAX_DOCUMENT_BYTES;
use crate::model::Animation;

/// The two fields every version of a document starts with.
#[derive(Deserialize)]
struct Header {
    format: String,
    version: u64,
}

/// The animation of the document `bytes`, of any known version, every rule checked.
///
/// # Errors
///
/// [`DocumentError::TooLarge`] above [`MAX_DOCUMENT_BYTES`], [`DocumentError::Malformed`] when it
/// is not a Life Pixel document, [`DocumentError::UnsupportedVersion`] for a version this build
/// does not know, or the first rule of the model the document breaks.
pub fn read_document(bytes: &[u8]) -> Result<Animation, DocumentError> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(DocumentError::TooLarge);
    }
    let header: Header = serde_json::from_slice(bytes).map_err(|_| DocumentError::Malformed)?;
    if header.format != DOCUMENT_FORMAT {
        return Err(DocumentError::Malformed);
    }
    match header.version {
        DOCUMENT_VERSION => json::read_v1(bytes),
        version => Err(DocumentError::UnsupportedVersion { version }),
    }
}
