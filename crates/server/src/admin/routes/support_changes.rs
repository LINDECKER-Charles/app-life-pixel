//! `PATCH /support-requests/{id}` and `POST /support-requests/{id}/messages`: the team changes a
//! request, and writes on it.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use super::OnRequest;
use super::responses::{AdminUnauthenticated, InvalidMessage, RequestNotFound};
use crate::admin::schema::{AdminSupportMessage, AdminSupportRequest, MessagePost, RequestPatch};
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::state::AppState;

/// Changes the status or the assignee of the request `id`, at least one of them. Only a new
/// status moves its `updatedAt`.
#[utoipa::path(
    patch,
    path = "/support-requests/{id}",
    tag = "support",
    operation_id = "updateSupportRequest",
    params(("id" = Uuid, Path, description = "The request's id")),
    request_body = RequestPatch,
    responses(
        (status = OK, description = "The request after the change", body = AdminSupportRequest),
        Malformed, AdminUnauthenticated, RequestNotFound
    )
)]
pub(super) async fn update_support_request(
    State(state): State<AppState>,
    request: OnRequest,
    Json(patch): Json<RequestPatch>,
) -> Result<Json<AdminSupportRequest>, Problem> {
    let update = patch.update(request.id)?;
    let changed = state.admin.update_request(&request.admin, update).await?;
    Ok(Json(changed.into()))
}

/// Adds a message of the team to the request `id`. A reply moves the request to
/// `waiting_for_user`, records the first response, and is emailed to the person as
/// `SupportReply`, with the link to `/support/{id}`; an internal note changes nothing else.
#[utoipa::path(
    post,
    path = "/support-requests/{id}/messages",
    tag = "support",
    operation_id = "postSupportMessage",
    params(("id" = Uuid, Path, description = "The request's id")),
    request_body = MessagePost,
    responses(
        (status = CREATED, description = "The message", body = AdminSupportMessage),
        Malformed, AdminUnauthenticated, RequestNotFound, InvalidMessage
    )
)]
pub(super) async fn post_support_message(
    State(state): State<AppState>,
    request: OnRequest,
    Json(post): Json<MessagePost>,
) -> Result<(StatusCode, Json<AdminSupportMessage>), Problem> {
    let message = state
        .admin
        .post_message(&request.admin, post.draft(request.id))
        .await?;
    Ok((StatusCode::CREATED, Json(message.into())))
}
