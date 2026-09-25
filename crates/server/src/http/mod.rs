//! The HTTP conventions every route shares: problems, request ids, security headers, the
//! client-version check, rate limits, the static app and the i18n endpoint. One line per module.

pub mod client_address;
pub mod client_version;
pub mod i18n;
pub mod problem;
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;
pub mod static_app;
