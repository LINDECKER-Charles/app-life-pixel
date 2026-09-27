//! A file held in memory, answered with its validators: a strong `ETag` from its SHA-256, and
//! `304 Not Modified` to a matching `If-None-Match`.

use std::fmt::Write;

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use sha2::{Digest, Sha256};

/// `Cache-Control` of a file that may change under the same name: revalidate every time.
pub const NO_CACHE: &str = "no-cache";

/// A file's bytes, with the headers it is served with.
#[derive(Clone, Debug)]
pub struct CachedFile {
    body: Bytes,
    content_type: HeaderValue,
    cache_control: HeaderValue,
    etag: HeaderValue,
}

impl CachedFile {
    /// The file `body`, served as `content_type` under `cache_control`.
    #[must_use]
    pub fn new(body: Bytes, content_type: &'static str, cache_control: &'static str) -> Self {
        let etag = strong_etag(&body);
        Self {
            body,
            content_type: HeaderValue::from_static(content_type),
            cache_control: HeaderValue::from_static(cache_control),
            etag,
        }
    }

    /// The answer to a `GET` or `HEAD` of the file carrying `headers`.
    #[must_use]
    pub fn respond(&self, method: &Method, headers: &HeaderMap) -> Response {
        let mut response = if self.is_fresh(headers) {
            StatusCode::NOT_MODIFIED.into_response()
        } else {
            self.full(method)
        };
        let response_headers = response.headers_mut();
        response_headers.insert(ETAG, self.etag.clone());
        response_headers.insert(CACHE_CONTROL, self.cache_control.clone());
        response
    }

    fn full(&self, method: &Method) -> Response {
        let body = if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(self.body.clone())
        };
        let mut response = Response::new(body);
        let headers = response.headers_mut();
        headers.insert(CONTENT_TYPE, self.content_type.clone());
        headers.insert(CONTENT_LENGTH, HeaderValue::from(self.body.len()));
        response
    }

    /// Whether the client's copy is current: `If-None-Match` lists this `ETag`, weakly
    /// compared as RFC 9110 asks, or is `*`.
    fn is_fresh(&self, headers: &HeaderMap) -> bool {
        let etag = self.etag.as_bytes();
        headers
            .get_all(IF_NONE_MATCH)
            .iter()
            .flat_map(|value| value.as_bytes().split(|byte| *byte == b','))
            .map(<[u8]>::trim_ascii)
            .any(|tag| tag == b"*" || tag.strip_prefix(b"W/").unwrap_or(tag) == etag)
    }
}

/// `"<SHA-256 in hexadecimal>"`.
fn strong_etag(body: &[u8]) -> HeaderValue {
    let digest = Sha256::digest(body);
    let mut etag = String::from("\"");
    for byte in digest.iter() {
        let _ = write!(etag, "{byte:02x}");
    }
    etag.push('"');
    HeaderValue::try_from(etag).unwrap_or_else(|_| HeaderValue::from_static("\"\""))
}
