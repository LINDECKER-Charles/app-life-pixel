//! Why the engine refuses a request: a stable code and its parameters, never a sentence — the
//! interface translates `errors.<code>`. The codes of `core` and `compiler` pass through as they
//! are; the engine adds its own below, and reuses the server's for a malformed request and for a
//! bug.

use life_pixel_compiler::ExportError;
use life_pixel_core::DocumentError;
use life_pixel_core::edit::EditError;
use serde::Serialize;
use serde_json::{Map, Value};

/// Every code this crate creates, each with its key `errors.<code>` in every catalogue.
pub const CODES: &[&str] = &[NO_DOCUMENT];

/// Every code this crate returns without creating it: the server's, already in every catalogue.
pub const REUSED_CODES: &[&str] = &[MALFORMED_REQUEST, INTERNAL_ERROR];

/// A command or a query that needs an animation, before `create` or `open`.
const NO_DOCUMENT: &str = "engine.no_document";
/// A request whose values do not have the shape of the engine interface.
const MALFORMED_REQUEST: &str = "request.malformed";
/// What only a bug causes: an encoder refusing a valid animation, a value that cannot cross.
const INTERNAL_ERROR: &str = "internal.error";

/// A refused request, serialized `{ code, params }` — the interface's `EngineError`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EngineError {
    /// The stable code: one of [`CODES`], of [`REUSED_CODES`], or of `core` and `compiler`.
    pub code: &'static str,
    /// The parameters of the code's message, named in camelCase.
    pub params: Map<String, Value>,
}

impl EngineError {
    /// No animation is open.
    #[must_use]
    pub fn no_document() -> Self {
        Self::without_params(NO_DOCUMENT)
    }

    /// The request's values do not have the shape the interface gives them.
    #[must_use]
    pub fn malformed_request() -> Self {
        Self::without_params(MALFORMED_REQUEST)
    }

    /// A failure only a bug causes.
    #[must_use]
    pub fn internal() -> Self {
        Self::without_params(INTERNAL_ERROR)
    }

    fn without_params(code: &'static str) -> Self {
        Self {
            code,
            params: Map::new(),
        }
    }
}

impl From<DocumentError> for EngineError {
    fn from(error: DocumentError) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}

impl From<EditError> for EngineError {
    fn from(error: EditError) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}

impl From<ExportError> for EngineError {
    fn from(error: ExportError) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}
