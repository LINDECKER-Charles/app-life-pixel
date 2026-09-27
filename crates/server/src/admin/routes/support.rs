//! `GET /support-requests`, `GET /support-requests/{id}` and
//! `GET /support-requests/{id}/screenshot`: the queue, one request's thread, its screenshot.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use super::OnRequest;
use super::responses::{AdminUnauthenticated, RequestNotFound, ScreenshotNotFound};
use crate::admin::schema::{AdminSupportRequest, AdminSupportThread, QueueQuery, Screenshot};
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::routes::library::schema::Page;
use crate::state::AppState;

/// The media type of a screenshot.
const PNG_MEDIA_TYPE: &str = "image/png";

/// A page of the queue, from the most recently updated request: each with its account's address
/// and its age.
#[utoipa::path(
    get,
    path = "/support-requests",
    tag = "support",
    operation_id = "listSupportRequests",
    params(QueueQuery),
    responses(
        (status = OK, description = "A page of the queue", body = Page<AdminSupportRequest>),
        Malformed, AdminUnauthenticated
    )
)]
pub(super) async fn list_support_requests(
    State(state): State<AppState>,
    Query(query): Query<QueueQuery>,
) -> Result<Json<Page<AdminSupportRequest>>, Problem> {
    let (filter, page) = query.filter_and_page()?;
    let page = state.admin.support_queue(filter, page).await?;
    Ok(Json(Page::of(page, AdminSupportRequest::from)))
}

/// The request `id`: its context and every message, the team's internal notes included.
#[utoipa::path(
    get,
    path = "/support-requests/{id}",
    tag = "support",
    operation_id = "getSupportRequest",
    params(("id" = Uuid, Path, description = "The request's id")),
    responses(
        (status = OK, description = "The request and its thread", body = AdminSupportThread),
        Malformed, AdminUnauthenticated, RequestNotFound
    )
)]
pub(super) async fn get_support_request(
    State(state): State<AppState>,
    request: OnRequest,
) -> Result<Json<AdminSupportThread>, Problem> {
    Ok(Json(state.admin.support_thread(request.id).await?.into()))
}

/// The screenshot of the request `id`, as the PNG it was stored as.
#[utoipa::path(
    get,
    path = "/support-requests/{id}/screenshot",
    tag = "support",
    operation_id = "getSupportScreenshot",
    params(("id" = Uuid, Path, description = "The request's id")),
    responses(
        (status = OK, description = "The PNG", body = Screenshot, content_type = "image/png"),
        Malformed, AdminUnauthenticated, ScreenshotNotFound
    )
)]
pub(super) async fn get_support_screenshot(
    State(state): State<AppState>,
    request: OnRequest,
) -> Result<Response, Problem> {
    let png = state.admin.support_screenshot(request.id).await?;
    let headers = [(CONTENT_TYPE, HeaderValue::from_static(PNG_MEDIA_TYPE))];
    Ok((StatusCode::OK, headers, png).into_response())
}
