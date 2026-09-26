//! `GET /account/export`: the account's data as a zip, built in a temporary file on a blocking
//! thread, then streamed as `life-pixel-export-<YYYY-MM-DD>.zip`.

use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio_util::io::ReaderStream;

use super::schema::AccountExport;
use crate::accounts::CurrentSession;
use crate::http::problem::Problem;
use crate::routes::library::responses::{RateLimited, Suspended, Unauthenticated};
use crate::state::AppState;

/// The media type of an export.
const ZIP_MEDIA_TYPE: &str = "application/zip";

/// The signed-in account's data: a zip laid out like a local library — the desktop app opens it
/// once unzipped —, plus `account.json` with its address, language, plan and sign-up date.
#[utoipa::path(
    get,
    path = "/account/export",
    tag = "account",
    operation_id = "exportAccount",
    security(("session" = [])),
    responses(
        (
            status = OK,
            description = "The zip, as an attachment named `life-pixel-export-<YYYY-MM-DD>.zip`",
            body = AccountExport,
            content_type = "application/zip",
            headers(("Content-Disposition" = String, description = "The file's name"))
        ),
        Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn export_account(
    State(state): State<AppState>,
    current: CurrentSession,
) -> Result<Response, Problem> {
    let id = current.session.account.id;
    let export = state.accounts.export_data(&state.library, id).await?;
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
