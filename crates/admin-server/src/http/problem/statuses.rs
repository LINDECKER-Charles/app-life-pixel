//! The codes the admin server answers with, and the table that gives each its HTTP status. The
//! relay passes the server's own problems through unchanged: they are not in this table. One
//! line per code, sorted.

use axum::http::StatusCode;

/// The codes, as constants.
pub mod codes {
    /// An unsafe request carrying the session cookie from another origin, or without the
    /// session's CSRF token.
    pub const ADMIN_CSRF: &str = "admin.csrf";
    /// A wrong address, password or TOTP code: which one is never said.
    pub const ADMIN_INVALID_CREDENTIALS: &str = "admin.invalid_credentials";
    /// No live session: signed out, idle 30 minutes, 8 hours old, or a disabled admin.
    pub const ADMIN_UNAUTHENTICATED: &str = "admin.unauthenticated";
    /// An unexpected failure, logged with the request id; nothing else is returned.
    pub const INTERNAL_ERROR: &str = "internal.error";
    /// A monitoring source that failed or answered what it should not; params `source`.
    pub const MONITORING_UNAVAILABLE: &str = "monitoring.unavailable";
    /// A monitoring source, or the environment's selector, left empty; params `source`.
    pub const MONITORING_NOT_CONFIGURED: &str = "monitoring.not_configured";
    /// Too many requests; params `retryAfterSeconds`, and the `Retry-After` header.
    pub const RATE_LIMIT_EXCEEDED: &str = "rate_limit.exceeded";
    /// A request that does not parse, or names an unknown environment or panel.
    pub const REQUEST_MALFORMED: &str = "request.malformed";
    /// A method the route does not accept.
    pub const REQUEST_METHOD_NOT_ALLOWED: &str = "request.method_not_allowed";
    /// A route that does not exist.
    pub const REQUEST_NOT_FOUND: &str = "request.not_found";
    /// A body above the route's limit; params `maxBytes`.
    pub const REQUEST_TOO_LARGE: &str = "request.too_large";
    /// A body in a media type the route does not read.
    pub const REQUEST_UNSUPPORTED_MEDIA_TYPE: &str = "request.unsupported_media_type";
    /// The admin database or the server's internal admin API does not answer.
    pub const SERVICE_UNAVAILABLE: &str = "service.unavailable";
}

/// Code to status: every code of this crate, each with its key `errors.<code>` in every
/// catalogue.
const STATUSES: &[(&str, StatusCode)] = &[
    (codes::ADMIN_CSRF, StatusCode::FORBIDDEN),
    (codes::ADMIN_INVALID_CREDENTIALS, StatusCode::UNAUTHORIZED),
    (codes::ADMIN_UNAUTHENTICATED, StatusCode::UNAUTHORIZED),
    (codes::INTERNAL_ERROR, StatusCode::INTERNAL_SERVER_ERROR),
    (
        codes::MONITORING_NOT_CONFIGURED,
        StatusCode::SERVICE_UNAVAILABLE,
    ),
    (codes::MONITORING_UNAVAILABLE, StatusCode::BAD_GATEWAY),
    (codes::RATE_LIMIT_EXCEEDED, StatusCode::TOO_MANY_REQUESTS),
    (codes::REQUEST_MALFORMED, StatusCode::BAD_REQUEST),
    (
        codes::REQUEST_METHOD_NOT_ALLOWED,
        StatusCode::METHOD_NOT_ALLOWED,
    ),
    (codes::REQUEST_NOT_FOUND, StatusCode::NOT_FOUND),
    (codes::REQUEST_TOO_LARGE, StatusCode::PAYLOAD_TOO_LARGE),
    (
        codes::REQUEST_UNSUPPORTED_MEDIA_TYPE,
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
    ),
    (codes::SERVICE_UNAVAILABLE, StatusCode::SERVICE_UNAVAILABLE),
];

/// Every code of the admin server.
pub const CODES: &[&str] = &[
    codes::ADMIN_CSRF,
    codes::ADMIN_INVALID_CREDENTIALS,
    codes::ADMIN_UNAUTHENTICATED,
    codes::INTERNAL_ERROR,
    codes::MONITORING_NOT_CONFIGURED,
    codes::MONITORING_UNAVAILABLE,
    codes::RATE_LIMIT_EXCEEDED,
    codes::REQUEST_MALFORMED,
    codes::REQUEST_METHOD_NOT_ALLOWED,
    codes::REQUEST_NOT_FOUND,
    codes::REQUEST_TOO_LARGE,
    codes::REQUEST_UNSUPPORTED_MEDIA_TYPE,
    codes::SERVICE_UNAVAILABLE,
];

/// The HTTP status of `code`, or `None` for a code the table does not know.
#[must_use]
pub fn status_of(code: &str) -> Option<StatusCode> {
    STATUSES
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, status)| *status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_a_status_and_a_message_in_every_catalogue() {
        let catalogues = concat!(env!("CARGO_MANIFEST_DIR"), "/../../i18n");
        for language in ["en", "fr"] {
            let path = format!("{catalogues}/{language}.json");
            let text = std::fs::read_to_string(&path).unwrap();
            let messages: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&text).unwrap();
            for code in CODES {
                assert!(status_of(code).is_some(), "{code} has no status");
                let key = format!("errors.{code}");
                assert!(messages.contains_key(&key), "{path} lacks {key}");
            }
        }
        assert_eq!(CODES.len(), STATUSES.len());
    }
}
