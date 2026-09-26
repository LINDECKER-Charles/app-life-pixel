//! The relay to the server's internal admin API (H10): `/users/…`, `/support-requests/…`,
//! `/metrics/product` and `/audit-log` go to `LPA_SERVER_ADMIN_API_URL` with the same method,
//! path and query, `Authorization: Bearer <LPA_SERVER_ADMIN_API_SECRET>`, and the signed-in
//! admin's `X-Admin-Id` and `X-Admin-Email`. Bodies are streamed both ways, and the server's
//! answers — its problems included — pass through unchanged. The admin server reads nothing of
//! the application's database itself.

use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{
    ACCEPT, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE, ETAG, RETRY_AFTER,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::Response;
use axum::routing::any;

use crate::http::problem::{Problem, codes};
use crate::http::request_id::REQUEST_ID_HEADER;
use crate::http::session::CurrentAdmin;
use crate::state::AppState;

/// The header naming the acting admin's id, as the internal admin API reads it.
pub const ADMIN_ID_HEADER: &str = "x-admin-id";
/// The header naming the acting admin's address.
pub const ADMIN_EMAIL_HEADER: &str = "x-admin-email";
/// The paths relayed, and those under them; the internal admin API's others are not.
pub const RELAYED_PATHS: [&str; 4] = [
    "/users",
    "/support-requests",
    "/metrics/product",
    "/audit-log",
];
/// How long the server has to accept a connection.
pub const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// How long the server may stay silent while it answers.
pub const RELAY_READ_TIMEOUT: Duration = Duration::from_secs(60);

/// The request headers passed on.
const REQUEST_HEADERS: [HeaderName; 2] = [ACCEPT, CONTENT_TYPE];
/// The response headers passed back.
const RESPONSE_HEADERS: [HeaderName; 6] = [
    CACHE_CONTROL,
    CONTENT_DISPOSITION,
    CONTENT_LENGTH,
    CONTENT_TYPE,
    ETAG,
    RETRY_AFTER,
];

/// The relayed routes, under `/api/admin/v1`.
pub fn router() -> Router<AppState> {
    RELAYED_PATHS.iter().fold(Router::new(), |router, path| {
        router
            .route(path, any(relay))
            .route(&format!("{path}/{{*rest}}"), any(relay))
    })
}

/// A client for the internal admin API.
///
/// # Errors
///
/// When the TLS backend cannot start.
pub fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(RELAY_CONNECT_TIMEOUT)
        .read_timeout(RELAY_READ_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

async fn relay(
    State(state): State<AppState>,
    current: CurrentAdmin,
    request: Request,
) -> Result<Response, Problem> {
    let path = request.uri().path();
    if has_dot_segment(path) {
        return Err(Problem::new(codes::REQUEST_NOT_FOUND));
    }
    let api = &state.config.server_admin_api;
    let mut url = api.url.join(path);
    if let Some(query) = request.uri().query() {
        url = format!("{url}?{query}");
    }
    let method = request.method().clone();
    let mut outgoing = state
        .relay_client
        .request(method.clone(), url)
        .headers(forwarded_headers(request.headers()))
        .bearer_auth(api.secret.expose())
        .headers(identity_headers(&current)?);
    if has_body(&method) {
        let stream = request.into_body().into_data_stream();
        outgoing = outgoing.body(reqwest::Body::wrap_stream(stream));
    }
    let answer = outgoing.send().await.map_err(|error| {
        tracing::error!(%error, "the internal admin API does not answer");
        Problem::new(codes::SERVICE_UNAVAILABLE)
    })?;
    passed_back(answer)
}

/// The request's headers the server reads, and its request id.
fn forwarded_headers(headers: &HeaderMap) -> HeaderMap {
    let mut forwarded = HeaderMap::new();
    for name in REQUEST_HEADERS.iter().chain([&REQUEST_ID_HEADER]) {
        if let Some(value) = headers.get(name) {
            forwarded.insert(name.clone(), value.clone());
        }
    }
    forwarded
}

/// `X-Admin-Id` and `X-Admin-Email`.
fn identity_headers(current: &CurrentAdmin) -> Result<HeaderMap, Problem> {
    let mut headers = HeaderMap::new();
    let id = HeaderValue::try_from(current.admin.id.hyphenated().to_string());
    let email = HeaderValue::try_from(current.admin.email.as_str());
    let (Ok(id), Ok(email)) = (id, email) else {
        tracing::error!(admin = %current.admin.id, "an admin's identity is not a header value");
        return Err(Problem::new(codes::INTERNAL_ERROR));
    };
    headers.insert(ADMIN_ID_HEADER, id);
    headers.insert(ADMIN_EMAIL_HEADER, email);
    Ok(headers)
}

/// The server's answer, its body streamed. A `401` means the server refused the admin server's
/// secret: a misconfiguration, never the admin's session, so the console gets `internal.error`.
fn passed_back(answer: reqwest::Response) -> Result<Response, Problem> {
    if answer.status() == StatusCode::UNAUTHORIZED {
        tracing::error!("the internal admin API refused LPA_SERVER_ADMIN_API_SECRET");
        return Err(Problem::new(codes::INTERNAL_ERROR));
    }
    let status = answer.status();
    let mut headers = HeaderMap::new();
    for name in &RESPONSE_HEADERS {
        if let Some(value) = answer.headers().get(name) {
            headers.insert(name.clone(), value.clone());
        }
    }
    let mut response = Response::new(Body::from_stream(answer.bytes_stream()));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    Ok(response)
}

fn has_body(method: &Method) -> bool {
    [Method::POST, Method::PUT, Method::PATCH, Method::DELETE].contains(method)
}

/// Whether `path` has a `.` or `..` segment, written plainly or percent-encoded: a URL parser
/// would resolve it, and the relay would leave the paths it serves.
fn has_dot_segment(path: &str) -> bool {
    path.split('/').any(|segment| {
        let decoded = segment.to_ascii_lowercase().replace("%2e", ".");
        decoded == "." || decoded == ".."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_segments_are_refused_however_written() {
        for path in [
            "/users/../environment",
            "/users/%2E%2e/x",
            "/users/.",
            "/users/.%2e",
        ] {
            assert!(has_dot_segment(path), "{path}");
        }
        assert!(!has_dot_segment("/users/0190f6a2-7c1e/export"));
        assert!(!has_dot_segment("/users/a.b"));
    }
}
