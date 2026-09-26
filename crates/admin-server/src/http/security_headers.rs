//! The security headers of docs/security-model.md, on every response of the admin server, and
//! `X-Robots-Tag`: the console is never indexed.

use axum::extract::State;
use axum::http::header::{
    CONTENT_SECURITY_POLICY, REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::Response;

/// The Content Security Policy: scripts strict, styles inline for Angular's component styles,
/// `blob:` for the screenshots and the CSV exports the console makes.
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
img-src 'self' data: blob:; connect-src 'self'; font-src 'self'; object-src 'none'; \
base-uri 'self'; form-action 'self'; frame-ancestors 'none'";

/// Two years of HTTPS only, subdomains included: sent when `LPA_PUBLIC_URL` is https.
const HSTS: &str = "max-age=63072000; includeSubDomains";
const NO_SNIFF: &str = "nosniff";
const NO_REFERRER: &str = "no-referrer";
const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");
const NO_PERMISSIONS: &str = "camera=(), microphone=(), geolocation=(), payment=()";
const OPENER_POLICY: HeaderName = HeaderName::from_static("cross-origin-opener-policy");
const SAME_ORIGIN: &str = "same-origin";
const ROBOTS_TAG: HeaderName = HeaderName::from_static("x-robots-tag");
const NOT_INDEXED: &str = "noindex, nofollow";

/// The headers every response gets, built once at start.
#[derive(Clone, Debug)]
pub struct SecurityHeaders(HeaderMap);

impl SecurityHeaders {
    /// The headers of a server whose public origin is, or is not, https.
    #[must_use]
    pub fn new(is_https: bool) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP));
        headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static(NO_SNIFF));
        headers.insert(REFERRER_POLICY, HeaderValue::from_static(NO_REFERRER));
        headers.insert(PERMISSIONS_POLICY, HeaderValue::from_static(NO_PERMISSIONS));
        headers.insert(OPENER_POLICY, HeaderValue::from_static(SAME_ORIGIN));
        headers.insert(ROBOTS_TAG, HeaderValue::from_static(NOT_INDEXED));
        if is_https {
            headers.insert(STRICT_TRANSPORT_SECURITY, HeaderValue::from_static(HSTS));
        }
        Self(headers)
    }
}

/// Response mapper: sets the security headers, over any a handler set.
pub async fn add(State(security): State<SecurityHeaders>, mut response: Response) -> Response {
    for (name, value) in &security.0 {
        response.headers_mut().insert(name, value.clone());
    }
    response
}
