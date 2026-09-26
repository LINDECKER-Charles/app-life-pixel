//! `GET /users/{id}/export`: the account's data export, H6's zip, as its owner would get it. The
//! read is audited before the zip is made.

use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio_util::io::ReaderStream;

use super::OnUser;
use super::responses::{AdminUnauthenticated, UserNotFound};
use crate::http::problem::Problem;
use crate::routes::account::schema::AccountExport;
use crate::routes::library::responses::Malformed;
use crate::state::AppState;

/// The media type of an export.
const ZIP_MEDIA_TYPE: &str = "application/zip";

/// The data of the account `id`: the zip of `GET /api/v1/account/export`.
#[utoipa::path(
    get,
    path = "/users/{id}/export",
    tag = "users",
    operation_id = "exportUser",
    params(("id" = Uuid, Path, description = "The account's id")),
    responses(
        (
            status = OK,
            description = "The zip, as an attachment named `life-pixel-export-<YYYY-MM-DD>.zip`",
            body = AccountExport,
            content_type = "application/zip",
            headers(("Content-Disposition" = String, description = "The file's name"))
        ),
        Malformed, AdminUnauthenticated, UserNotFound
    )
)]
pub(super) async fn export_user(
    State(state): State<AppState>,
    user: OnUser,
) -> Result<Response, Problem> {
    let export = state.admin.export_user(&user.admin, user.id).await?;
    let disposition = format!("attachment; filename=\"{}\"", export.file_name());
    let disposition =
        HeaderValue::try_from(disposition).map_err(|error| Problem::internal(&error))?;
    let headers = [
        (CONTENT_TYPE, HeaderValue::from_static(ZIP_MEDIA_TYPE)),
        (CONTENT_DISPOSITION, disposition),
        (CONTENT_LENGTH, HeaderValue::from(export.bytes)),
    ];
    let file = tokio::fs::File::from_std(export.file);
    let body = Body::from_stream(ReaderStream::new(file));
    Ok((StatusCode::OK, headers, body).into_response())
}
