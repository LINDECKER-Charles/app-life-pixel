//! The body of the routes that write a document: `application/vnd.life-pixel.animation+json`, at
//! most `MAX_DOCUMENT_BYTES` — the routes' own body limit, above the API's.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_TYPE;
use life_pixel_core::DocumentError;
use life_pixel_core::limits::MAX_DOCUMENT_BYTES;

use crate::http::problem::{Problem, codes};

/// The media type of a document.
pub const DOCUMENT_MEDIA_TYPE: &str = "application/vnd.life-pixel.animation+json";

/// A document's bytes, not yet parsed: `service` checks them.
pub struct DocumentBody(pub Bytes);

impl<S: Send + Sync> FromRequest<S> for DocumentBody {
    type Rejection = Problem;

    async fn from_request(request: Request, _state: &S) -> Result<Self, Problem> {
        if !is_document(request.headers()) {
            return Err(Problem::new(codes::REQUEST_UNSUPPORTED_MEDIA_TYPE));
        }
        // A body that cannot be read whole within the limit is refused as too large: the other
        // failures are the client's connection, which reads no answer.
        let body = axum::body::to_bytes(request.into_body(), MAX_DOCUMENT_BYTES).await;
        body.map(Self)
            .map_err(|_| Problem::from(DocumentError::TooLarge))
    }
}

/// Whether the request's `Content-Type` is a document's, whatever its parameters.
fn is_document(headers: &HeaderMap) -> bool {
    let media_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next());
    media_type.is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case(DOCUMENT_MEDIA_TYPE))
}
