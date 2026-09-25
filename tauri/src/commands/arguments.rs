//! The commands' arguments decoded here rather than by Tauri, so that a bad one answers
//! `request.malformed` as every other failure answers a code.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use life_pixel_service::{Cursor, PageRequest};
use uuid::Uuid;

use crate::errors::CommandError;

/// The id of kind `T` in `text`, `argument` naming it in the log.
///
/// # Errors
///
/// `request.malformed` when `text` is not a UUID.
pub fn id<T: From<Uuid>>(argument: &str, text: &str) -> Result<T, CommandError> {
    let uuid = Uuid::try_parse(text).map_err(|error| CommandError::malformed(argument, error))?;
    Ok(T::from(uuid))
}

/// The page `cursor` and `limit` ask for; a limit beyond the largest page gets the largest page.
///
/// # Errors
///
/// `request.malformed` when the cursor does not decode.
pub fn page(cursor: Option<String>, limit: Option<u32>) -> Result<PageRequest, CommandError> {
    let cursor = cursor
        .map(|text| text.parse::<Cursor>())
        .transpose()
        .map_err(|error| CommandError::malformed("cursor", error))?;
    let limit = limit.map(|limit| u16::try_from(limit).unwrap_or(u16::MAX));
    Ok(PageRequest::new(cursor, limit))
}

/// The bytes of `text`, in base64, `argument` naming it in the log.
///
/// # Errors
///
/// `request.malformed` when `text` is not base64.
pub fn bytes(argument: &str, text: &str) -> Result<Vec<u8>, CommandError> {
    STANDARD
        .decode(text)
        .map_err(|error| CommandError::malformed(argument, error))
}

/// `bytes` in base64.
#[must_use]
pub fn base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}
