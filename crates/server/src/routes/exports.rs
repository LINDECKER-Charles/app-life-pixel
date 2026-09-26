//! `GET /exports/{link}` (A3): the file a signed link of the MCP `export` tool names, compiled
//! again from the version it names and sent as an attachment. The link is the credential: no
//! session is needed. Nothing is stored or cached.

use std::time::Instant;

use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use life_pixel_service::mcp::McpError;
use utoipa::{IntoResponses, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::problem::{Problem, ProblemDocument};
use crate::mcp::metrics::record_export;
use crate::routes::library::responses::RateLimited;
use crate::state::AppState;

/// The routes under the `api` rate limit: one line per path.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(download_export))
}

/// An export file.
#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct ExportFileBody(pub Vec<u8>);

/// `410`: the link was tampered with or expired, or its animation changed since.
#[derive(IntoResponses)]
#[response(
    status = GONE,
    description = "`export.link_invalid`",
    content_type = "application/problem+json"
)]
pub struct LinkInvalid(pub ProblemDocument);

/// The file the link names, compiled from the version it names; a link lives 15 minutes.
#[utoipa::path(
    get,
    path = "/exports/{link}",
    tag = "exports",
    operation_id = "downloadExport",
    params(("link" = String, Path, description = "The signed link the `export` tool answered")),
    responses(
        (
            status = OK,
            description = "The file, as an attachment under its name",
            body = ExportFileBody,
            content_type = "application/octet-stream",
            headers(("Content-Disposition" = String, description = "The file's name"))
        ),
        LinkInvalid, RateLimited
    )
)]
pub(super) async fn download_export(
    State(state): State<AppState>,
    Path(link): Path<String>,
) -> Result<Response, Problem> {
    let format = state.mcp.link_format(&link).ok_or(McpError::LinkInvalid)?;
    let start = Instant::now();
    let file = state.mcp.downloads().download(&link).await;
    let size = file.as_ref().ok().map(|file| file.bytes.len());
    record_export(format, (size, start.elapsed()));
    let file = file?;
    let disposition = format!("attachment; filename=\"{}\"", safe_name(&file.name));
    let disposition =
        HeaderValue::try_from(disposition).map_err(|error| Problem::internal(&error))?;
    let headers = [
        (CONTENT_TYPE, HeaderValue::from_static(file.media_type)),
        (CONTENT_DISPOSITION, disposition),
        (CACHE_CONTROL, HeaderValue::from_static("no-store")),
    ];
    Ok((StatusCode::OK, headers, file.bytes).into_response())
}

/// `name` with only the characters a quoted `filename` takes as they are.
fn safe_name(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}
