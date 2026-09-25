//! Body limits: 1 MiB under `/api/v1` and `/mcp`, and the larger ones a route declares.

use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::StatusCode;
use axum::middleware::{Next, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use utoipa_axum::router::OpenApiRouter;

use super::{Problem, codes, is_problem};

/// The body limit of every API route that declares no larger one: 1 MiB.
pub const API_BODY_LIMIT_BYTES: usize = 1024 * 1024;

/// A body limit on a group of routes: the extractors stop reading beyond `max_bytes`, and the
/// client gets `request.too_large` with `maxBytes`. A limit set on a route group overrides the
/// one set around it.
pub trait BodyLimit {
    /// The routes, with their bodies capped at `max_bytes`.
    #[must_use]
    fn body_limit(self, max_bytes: usize) -> Self;
}

impl<S: Clone + Send + Sync + 'static> BodyLimit for Router<S> {
    fn body_limit(self, max_bytes: usize) -> Self {
        self.layer(from_fn_with_state(max_bytes, too_large))
            .layer(DefaultBodyLimit::max(max_bytes))
    }
}

impl<S: Clone + Send + Sync + 'static> BodyLimit for OpenApiRouter<S> {
    fn body_limit(self, max_bytes: usize) -> Self {
        self.layer(from_fn_with_state(max_bytes, too_large))
            .layer(DefaultBodyLimit::max(max_bytes))
    }
}

/// Middleware: the extractors' `413` becomes `request.too_large` with the limit.
async fn too_large(State(max_bytes): State<usize>, request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if response.status() != StatusCode::PAYLOAD_TOO_LARGE || is_problem(&response) {
        return response;
    }
    Problem::new(codes::REQUEST_TOO_LARGE)
        .with_param("maxBytes", max_bytes)
        .into_response()
}
