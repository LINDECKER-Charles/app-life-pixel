//! Why an export is refused: a stable code and its parameters, never a sentence for the user —
//! the interface translates `errors.<code>`.

use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MAX_SIDE, EXPORT_MIN_SCALE};
use serde_json::{Map, Value, json};
use thiserror::Error;

/// Every code of this crate, each with its key `errors.<code>` in every catalogue.
pub const CODES: &[&str] = &["export.scale", "export.tag_not_found", "export.too_large"];

/// The code of an encoder failure: the server's, since only a bug causes one.
const INTERNAL_ERROR: &str = "internal.error";

/// An export that cannot be made.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ExportError {
    /// A scale out of [`EXPORT_MIN_SCALE`] to [`EXPORT_MAX_SCALE`].
    #[error("scale out of {EXPORT_MIN_SCALE} to {EXPORT_MAX_SCALE}")]
    Scale,
    /// An image wider or taller than [`EXPORT_MAX_SIDE`] pixels once scaled.
    #[error("image side above {EXPORT_MAX_SIDE} pixels")]
    TooLarge,
    /// No tag of that name in the animation.
    #[error("no tag named {name:?}")]
    TagNotFound {
        /// The tag's name, as asked.
        name: String,
    },
    /// An encoder refused its input, which a valid animation within the limits never causes.
    #[error("encoder failure: {0}")]
    Encoding(String),
}

impl ExportError {
    /// The stable code of this error: one of [`CODES`], or `internal.error` for
    /// [`ExportError::Encoding`].
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Scale => "export.scale",
            Self::TooLarge => "export.too_large",
            Self::TagNotFound { .. } => "export.tag_not_found",
            Self::Encoding(_) => INTERNAL_ERROR,
        }
    }

    /// The parameters of this error, named in camelCase as the catalogues' messages use them.
    #[must_use]
    pub fn params(&self) -> Map<String, Value> {
        let params = match self {
            Self::Scale => json!({ "min": EXPORT_MIN_SCALE, "max": EXPORT_MAX_SCALE }),
            Self::TooLarge => json!({ "maxSide": EXPORT_MAX_SIDE }),
            Self::TagNotFound { name } => json!({ "name": name }),
            Self::Encoding(_) => json!({}),
        };
        match params {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }

    /// An encoder's failure, kept as its message.
    pub(crate) fn encoding(error: impl std::fmt::Display) -> Self {
        Self::Encoding(error.to_string())
    }
}
