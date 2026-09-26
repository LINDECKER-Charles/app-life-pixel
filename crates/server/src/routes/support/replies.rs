//! `POST /support-requests/{id}/messages`: the account replies to one of its requests.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use super::OwnRequest;
use super::responses::{InvalidReply, RequestClosed, RequestNotFound};
use super::schema::{SupportReplyRequest, SupportRequestMessage};
use crate::http::problem::Problem;
use crate::routes::library::responses::{Forbidden, Malformed, RateLimited, Unauthenticated};
use crate::state::AppState;

/// Adds a reply to the request `id`: a request waiting for the person goes back to
/// `in_progress`; a closed one refuses it.
#[utoipa::path(
    post,
    path = "/support-requests/{id}/messages",
    tag = "support",
    operation_id = "replyToSupportRequest",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The request's id")),
    request_body = SupportReplyRequest,
    responses(
        (status = CREATED, description = "The reply", body = SupportRequestMessage),
        Malformed, Unauthenticated, Forbidden, RequestNotFound, RequestClosed, InvalidReply,
        RateLimited
    )
)]
pub(super) async fn reply_to_support_request(
    State(state): State<AppState>,
    own: OwnRequest,
    Json(reply): Json<SupportReplyRequest>,
) -> Result<(StatusCode, Json<SupportRequestMessage>), Problem> {
    let message = state
        .support
        .reply(own.account, (own.id, &reply.body))
        .await?;
    Ok((StatusCode::CREATED, Json(message.into())))
}
