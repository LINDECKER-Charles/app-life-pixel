//! Errors as stable codes with parameters, never sentences: the interface translates
//! `errors.<code>`, and the server maps each code to its HTTP status.

use life_pixel_core::DocumentError;
use serde_json::{Map, Value};
use thiserror::Error;

/// Every code of this crate, each with its key `errors.<code>` in every catalogue. The codes of
/// `core` travel through the service unchanged and stay in `life_pixel_core::error::CODES`.
pub const CODES: &[&str] = &[
    "account.language",
    "admin.reason_length",
    "admin.screenshot_not_found",
    "admin.user_not_found",
    "auth.account_suspended",
    "auth.current_password",
    "auth.email_invalid",
    "auth.email_taken",
    "auth.invalid_credentials",
    "auth.password_length",
    "auth.token_invalid",
    "auth.unauthenticated",
    "document.version_conflict",
    "draw.too_many_operations",
    "edit.palette_in_use",
    "library.animation_not_found",
    "library.project_not_found",
    "library.unavailable",
    "library.unsupported_version",
    "preview.too_large",
    "quota.storage_exceeded",
    "request.malformed",
    "service.unavailable",
    "support.category",
    "support.message_length",
    "support.request_closed",
    "support.request_not_found",
    "support.screenshot",
];

/// An error the product can show: a stable code and its parameters.
pub trait Coded {
    /// The stable code, dot-separated `snake_case` segments, domain first.
    fn code(&self) -> &'static str;
    /// The parameters of the code's message, with camelCase keys.
    fn params(&self) -> Map<String, Value>;
}

/// Any coded error, once its origin no longer matters: what MCP and the desktop return.
#[derive(Clone, Debug, PartialEq, Error)]
#[error("{code}")]
pub struct CodedError {
    /// The stable code.
    pub code: &'static str,
    /// The parameters of the code's message, with camelCase keys.
    pub params: Map<String, Value>,
}

impl CodedError {
    /// The code and parameters of `error`.
    #[must_use]
    pub fn of(error: &(impl Coded + ?Sized)) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}

impl Coded for CodedError {
    fn code(&self) -> &'static str {
        self.code
    }

    fn params(&self) -> Map<String, Value> {
        self.params.clone()
    }
}

impl Coded for DocumentError {
    fn code(&self) -> &'static str {
        DocumentError::code(self)
    }

    fn params(&self) -> Map<String, Value> {
        DocumentError::params(self)
    }
}

impl From<DocumentError> for CodedError {
    fn from(error: DocumentError) -> Self {
        Self::of(&error)
    }
}

/// The parameters `pairs`, as a map.
pub(crate) fn params<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}
