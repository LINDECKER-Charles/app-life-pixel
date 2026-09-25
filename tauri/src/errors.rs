//! Why a command failed: a stable code and its parameters, serialized as the API's problems carry
//! them, so that the app translates `errors.<code>` whichever back-end answered.

use std::fmt::Display;
use std::path::Path;

use life_pixel_service::Coded;
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

/// An export could not be written where the person chose; parameter `path`.
pub const EXPORT_WRITE_FAILED: &str = "export.write_failed";
/// An argument does not decode: an id, a cursor, base64, a file name, a language.
pub const REQUEST_MALFORMED: &str = "request.malformed";
/// The settings file or a worker failed; the detail is logged.
pub const SERVICE_UNAVAILABLE: &str = "service.unavailable";

/// The codes this crate creates, each with its key `errors.<code>` in every catalogue; the others
/// come from `service` and `core`.
pub const CODES: &[&str] = &[EXPORT_WRITE_FAILED];

/// The error every command returns: `{ "code": "…", "params": { … } }`.
#[derive(Clone, Debug, PartialEq, Serialize, Error)]
#[error("{code}")]
pub struct CommandError {
    /// The stable code.
    pub code: &'static str,
    /// The parameters of the code's message, with camelCase keys.
    pub params: Map<String, Value>,
}

impl CommandError {
    /// The error `code`, without parameters.
    #[must_use]
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
            params: Map::new(),
        }
    }

    /// `request.malformed`, logging what did not decode.
    pub fn malformed(argument: &str, reason: impl Display) -> Self {
        tracing::warn!(argument, %reason, "malformed command argument");
        Self::new(REQUEST_MALFORMED)
    }

    /// `service.unavailable`, logging why.
    pub fn unavailable(reason: impl Display) -> Self {
        tracing::error!(%reason, "desktop command failed");
        Self::new(SERVICE_UNAVAILABLE)
    }

    /// `export.write_failed` for `path`, logging why.
    pub fn export_write_failed(path: &Path, reason: impl Display) -> Self {
        tracing::warn!(path = %path.display(), %reason, "export not written");
        let mut error = Self::new(EXPORT_WRITE_FAILED);
        let path = Value::from(path.display().to_string());
        error.params.insert("path".to_owned(), path);
        error
    }
}

impl<E: Coded> From<E> for CommandError {
    fn from(error: E) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}

#[cfg(test)]
mod tests {
    use life_pixel_service::library::LibraryError;
    use serde_json::json;

    use super::*;

    #[test]
    fn an_error_serializes_as_a_problem_carries_it() {
        let error = CommandError::from(LibraryError::VersionConflict { current: 4 });
        let json = serde_json::to_value(error).unwrap();
        assert_eq!(
            json,
            json!({ "code": "document.version_conflict", "params": { "current": 4 } })
        );
    }

    #[test]
    fn every_code_is_in_every_catalogue() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../i18n");
        for language in ["en", "fr"] {
            let text = std::fs::read_to_string(root.join(format!("{language}.json"))).unwrap();
            let catalogue: Map<String, Value> = serde_json::from_str(&text).unwrap();
            for code in CODES
                .iter()
                .chain([&REQUEST_MALFORMED, &SERVICE_UNAVAILABLE])
            {
                let key = format!("errors.{code}");
                assert!(
                    catalogue.contains_key(&key),
                    "{key} missing from {language}.json"
                );
            }
        }
    }
}
