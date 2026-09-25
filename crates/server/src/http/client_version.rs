//! The client-version check: an app older than its platform's minimum must update.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use semver::Version;

use super::problem::{Problem, codes};
use crate::config::ClientVersions;

/// `Life-Pixel-Client: <platform>/<version>`, such as `web/1.0.0` or `desktop/1.0.0`.
pub const CLIENT_HEADER: HeaderName = HeaderName::from_static("life-pixel-client");

/// Middleware: answers `client.update_required` with the `minimum` to a client below its
/// platform's minimum. A request without the header — a script's — passes; a header that does
/// not parse is `request.malformed`.
///
/// # Errors
///
/// The problem of a malformed header or an outdated client.
pub async fn require_supported(
    State(minimums): State<ClientVersions>,
    request: Request,
    next: Next,
) -> Result<Response, Problem> {
    if let Some(value) = request.headers().get(CLIENT_HEADER) {
        check(value, &minimums)?;
    }
    Ok(next.run(request).await)
}

fn check(value: &HeaderValue, minimums: &ClientVersions) -> Result<(), Problem> {
    let (platform, version) = parse(value).ok_or_else(|| Problem::new(codes::REQUEST_MALFORMED))?;
    match minimums.minimum(platform) {
        Some(minimum) if version < *minimum => {
            Err(Problem::new(codes::CLIENT_UPDATE_REQUIRED)
                .with_param("minimum", minimum.to_string()))
        }
        _ => Ok(()),
    }
}

/// The platform and version of a `Life-Pixel-Client` value.
fn parse(value: &HeaderValue) -> Option<(&str, Version)> {
    let (platform, version) = value.to_str().ok()?.split_once('/')?;
    let version = Version::parse(version).ok()?;
    (!platform.is_empty()).then_some((platform, version))
}
