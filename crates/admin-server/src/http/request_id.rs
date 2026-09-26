//! The request id: kept from a trusted proxy, generated otherwise, returned, and attached to
//! the request's span and logs.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use super::client_address::peer_address;
use crate::config::TrustedProxies;

/// The header carrying the request id, both ways.
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// The longest request id kept from a proxy.
const MAX_REQUEST_ID_CHARS: usize = 128;

/// The id of the current request, in its extensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(HeaderValue);

impl RequestId {
    /// The id, as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap_or_default()
    }
}

/// Middleware: gives the request its id, and the response its `X-Request-Id`.
pub async fn assign(
    State(proxies): State<TrustedProxies>,
    mut request: Request,
    next: Next,
) -> Response {
    let peer = peer_address(request.extensions());
    let forwarded = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .filter(|_| proxies.contains(peer))
        .filter(|id| is_valid(id));
    let id = forwarded.cloned().unwrap_or_else(generate);
    request.headers_mut().insert(REQUEST_ID_HEADER, id.clone());
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    response.headers_mut().insert(REQUEST_ID_HEADER, id);
    response
}

/// A new id: a UUIDv7, ordered by time.
fn generate() -> HeaderValue {
    let id = Uuid::now_v7().hyphenated().to_string();
    HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("unknown"))
}

/// Whether a proxy's id is safe to keep: short, and letters, digits, `-`, `_`, `.` or `:` only,
/// so that it cannot forge a log line.
fn is_valid(id: &HeaderValue) -> bool {
    let bytes = id.as_bytes();
    let is_allowed = |byte: &u8| byte.is_ascii_alphanumeric() || b"-_.:".contains(byte);
    !bytes.is_empty() && bytes.len() <= MAX_REQUEST_ID_CHARS && bytes.iter().all(is_allowed)
}
