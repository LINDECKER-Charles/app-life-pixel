//! `POST /support-requests`: Help → Contact.

use axum::Json;
use axum::extract::State;
use axum::extract::multipart::{Multipart, MultipartRejection};
use axum::http::StatusCode;
use life_pixel_service::support::SupportError;

use super::Requester;
use super::form::{SupportRequestForm, read_submission};
use super::responses::{CreationRateLimited, FormTooLarge, InvalidRequest, NotAForm};
use super::schema::SupportRequestThread;
use crate::http::problem::{Problem, codes};
use crate::http::rate_limit::{Policy, RateKey};
use crate::routes::library::responses::{Forbidden, Malformed, Unauthenticated};
use crate::state::AppState;

/// Sends a support request: its message becomes its first message, its screenshot is decoded
/// under the limits and re-encoded as PNG, and the context is kept for the team. Records
/// `support_request_created`.
#[utoipa::path(
    post,
    path = "/support-requests",
    tag = "support",
    operation_id = "createSupportRequest",
    security(("session" = [], "csrf" = [])),
    request_body(content = SupportRequestForm, content_type = "multipart/form-data"),
    responses(
        (status = CREATED, description = "The request", body = SupportRequestThread),
        Malformed, Unauthenticated, Forbidden, FormTooLarge, NotAForm, InvalidRequest,
        CreationRateLimited
    )
)]
pub(super) async fn create_support_request(
    State(state): State<AppState>,
    Requester(account): Requester,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, Json<SupportRequestThread>), Problem> {
    state
        .rate_limits
        .check(Policy::SupportCreate, &RateKey::Account(account))?;
    let multipart = multipart.map_err(|_| Problem::new(codes::REQUEST_UNSUPPORTED_MEDIA_TYPE))?;
    let submission = read_submission(multipart).await?;
    let created = state.support.create(account, submission).await;
    let thread = created.map_err(|error: SupportError| Problem::from(error))?;
    Ok((StatusCode::CREATED, Json(thread.into())))
}
