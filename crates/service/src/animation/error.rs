//! Why an agent's use case failed, as a code the interface translates.

use life_pixel_compiler::ExportError;
use life_pixel_core::DocumentError;
use life_pixel_core::edit::EditError;
use life_pixel_core::limits::{DRAW_MAX_OPERATIONS, PREVIEW_MAX_BYTES, PREVIEW_MAX_SIDE};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::error::{Coded, CodedError, params};
use crate::library::LibraryError;

/// The error of an [`AnimationEditing`](super::AnimationEditing) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum EditingError {
    /// The library refused: a missing animation, a conflict, the quota, an invalid document.
    #[error(transparent)]
    Library(#[from] LibraryError),
    /// `core` refused an operation: its code.
    #[error(transparent)]
    Edit(#[from] EditError),
    /// `compiler` refused an export: its code.
    #[error(transparent)]
    Export(#[from] ExportError),
    /// `draw.too_many_operations`: more than [`DRAW_MAX_OPERATIONS`] in one call.
    #[error("more than {DRAW_MAX_OPERATIONS} drawing operations")]
    TooManyOperations,
    /// `edit.palette_in_use`: a palette entry the new palette drops is still painted.
    #[error("palette entry {index} is still used")]
    PaletteInUse {
        /// The lowest entry dropped that a cel still uses.
        index: usize,
    },
    /// `preview.too_large`: above [`PREVIEW_MAX_SIDE`] pixels a side or [`PREVIEW_MAX_BYTES`].
    #[error("preview above {PREVIEW_MAX_SIDE} pixels a side or {PREVIEW_MAX_BYTES} bytes")]
    PreviewTooLarge,
}

impl From<DocumentError> for EditingError {
    fn from(error: DocumentError) -> Self {
        Self::Library(error.into())
    }
}

impl Coded for EditingError {
    fn code(&self) -> &'static str {
        match self {
            Self::Library(error) => error.code(),
            Self::Edit(error) => error.code(),
            Self::Export(error) => error.code(),
            Self::TooManyOperations => "draw.too_many_operations",
            Self::PaletteInUse { .. } => "edit.palette_in_use",
            Self::PreviewTooLarge => "preview.too_large",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::Library(error) => error.params(),
            Self::Edit(error) => error.params(),
            Self::Export(error) => error.params(),
            Self::TooManyOperations => params([("max", DRAW_MAX_OPERATIONS.into())]),
            Self::PaletteInUse { index } => params([("index", (*index).into())]),
            Self::PreviewTooLarge => params([("maxSide", PREVIEW_MAX_SIDE.into())]),
        }
    }
}

impl From<EditingError> for CodedError {
    fn from(error: EditingError) -> Self {
        Self::of(&error)
    }
}
