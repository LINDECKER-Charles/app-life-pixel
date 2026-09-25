//! The framework's own error answers — unknown routes and methods, rejected bodies, panics —
//! turned into problems, so that every API error is `application/problem+json`.

use std::any::Any;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header::ALLOW;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::{Problem, codes, is_problem};

/// The answer to a route that does not exist.
pub async fn not_found() -> Problem {
    Problem::new(codes::REQUEST_NOT_FOUND)
}

/// Middleware: replaces an error answer that is not a problem with the problem of its status,
/// keeping `Allow` on a `405`.
pub async fn ensure_problem(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if is_problem(&response) {
        return response;
    }
    let Some(code) = code_for(response.status()) else {
        return response;
    };
    let mut problem = Problem::new(code).into_response();
    if let Some(allow) = response.headers().get(ALLOW) {
        problem.headers_mut().insert(ALLOW, allow.clone());
    }
    problem
}

/// The answer to a handler that panicked: logged with the request id, `internal.error` sent.
pub fn panic_problem(panic: Box<dyn Any + Send + 'static>) -> Response {
    let message = panic
        .downcast_ref::<&str>()
        .map(ToString::to_string)
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_default();
    tracing::error!(panic = %message, "a request handler panicked");
    Problem::new(codes::INTERNAL_ERROR).into_response()
}

/// The code of an error status the framework answers with; `None` for a success, or for a
/// status only a problem should carry.
fn code_for(status: StatusCode) -> Option<&'static str> {
    match status {
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
            Some(codes::REQUEST_MALFORMED)
        }
        StatusCode::NOT_FOUND => Some(codes::REQUEST_NOT_FOUND),
        StatusCode::METHOD_NOT_ALLOWED => Some(codes::REQUEST_METHOD_NOT_ALLOWED),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => Some(codes::REQUEST_UNSUPPORTED_MEDIA_TYPE),
        status if status.is_server_error() => Some(codes::INTERNAL_ERROR),
        _ => None,
    }
}
