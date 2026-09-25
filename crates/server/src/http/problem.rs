//! Problems: every error answers `application/problem+json`, with a stable code and its
//! parameters. A [`Coded`] error becomes a [`Problem`] through the code-to-status table.

mod body_limit;
mod document;
mod fallback;
mod statuses;

use axum::Json;
use axum::http::header::{CONTENT_TYPE, RETRY_AFTER};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use life_pixel_service::Coded;
use serde_json::{Map, Value};

pub use body_limit::{API_BODY_LIMIT_BYTES, BodyLimit};
pub use document::ProblemDocument;
pub use fallback::{ensure_problem, not_found, panic_problem};
pub use statuses::{CODES, codes, status_of};

/// The media type of every error.
pub const PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

/// The prefix of a problem's `type`, before its code.
const PROBLEM_TYPE_PREFIX: &str = "urn:life-pixel:problem:";

/// An error on its way to the client.
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    /// The status of the code.
    pub status: StatusCode,
    /// The stable code.
    pub code: &'static str,
    /// The parameters of the code's message, with camelCase keys.
    pub params: Map<String, Value>,
    /// Seconds before trying again, sent as `Retry-After`.
    pub retry_after: Option<u32>,
}

impl Problem {
    /// The problem of `code`, with the status the table gives it and no parameter. A code the
    /// table does not know is a bug: it is logged, and answered as `internal.error`.
    #[must_use]
    pub fn new(code: &'static str) -> Self {
        let Some(status) = status_of(code) else {
            tracing::error!(code, "a code without an HTTP status");
            return Self::new(codes::INTERNAL_ERROR);
        };
        Self {
            status,
            code,
            params: Map::new(),
            retry_after: None,
        }
    }

    /// The problem of a coded error, its parameters included.
    #[must_use]
    pub fn of(error: &(impl Coded + ?Sized)) -> Self {
        Self {
            params: error.params(),
            ..Self::new(error.code())
        }
    }

    /// An unexpected failure: `error` is logged, with the request id of the current span, and
    /// the client only learns `internal.error`.
    #[must_use]
    pub fn internal(error: &dyn std::error::Error) -> Self {
        tracing::error!(error = %error, "internal error");
        Self::new(codes::INTERNAL_ERROR)
    }

    /// The problem, with the parameter `name` set to `value`.
    #[must_use]
    pub fn with_param(mut self, name: &str, value: impl Into<Value>) -> Self {
        self.params.insert(name.to_owned(), value.into());
        self
    }

    /// The problem, telling the client to wait `seconds` before trying again.
    #[must_use]
    pub fn with_retry_after(mut self, seconds: u32) -> Self {
        self.retry_after = Some(seconds);
        self
    }

    /// The body the client receives.
    #[must_use]
    pub fn document(&self) -> ProblemDocument {
        ProblemDocument {
            problem_type: format!("{PROBLEM_TYPE_PREFIX}{}", self.code),
            status: self.status.as_u16(),
            code: self.code.to_owned(),
            params: self.params.clone(),
        }
    }
}

impl<E: Coded> From<E> for Problem {
    fn from(error: E) -> Self {
        Self::of(&error)
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let content_type = HeaderValue::from_static(PROBLEM_CONTENT_TYPE);
        let mut response = (
            self.status,
            [(CONTENT_TYPE, content_type)],
            Json(self.document()),
        )
            .into_response();
        if let Some(seconds) = self.retry_after {
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

/// Whether `response` already carries a problem.
pub(crate) fn is_problem(response: &Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .is_some_and(|value| value.as_bytes() == PROBLEM_CONTENT_TYPE.as_bytes())
}
