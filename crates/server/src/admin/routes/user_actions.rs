//! `POST /users/{id}/suspend`, `POST /users/{id}/reactivate` and `DELETE /users/{id}`: what an
//! admin does to an account, each for a reason the audit log keeps.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use super::OnUser;
use super::responses::{AdminUnauthenticated, InvalidReason, UserNotFound};
use crate::admin::schema::ReasonBody;
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::state::AppState;

/// Suspends the account `id`: it may not sign in, and its sessions end.
#[utoipa::path(
    post,
    path = "/users/{id}/suspend",
    tag = "users",
    operation_id = "suspendUser",
    params(("id" = Uuid, Path, description = "The account's id")),
    request_body = ReasonBody,
    responses(
        (status = NO_CONTENT, description = "Suspended"),
        Malformed, AdminUnauthenticated, UserNotFound, InvalidReason
    )
)]
pub(super) async fn suspend_user(
    State(state): State<AppState>,
    user: OnUser,
    Json(body): Json<ReasonBody>,
) -> Result<StatusCode, Problem> {
    state
        .admin
        .suspend_user(&user.admin, (user.id, &body.reason))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reactivates the account `id`: it may sign in again.
#[utoipa::path(
    post,
    path = "/users/{id}/reactivate",
    tag = "users",
    operation_id = "reactivateUser",
    params(("id" = Uuid, Path, description = "The account's id")),
    request_body = ReasonBody,
    responses(
        (status = NO_CONTENT, description = "Reactivated"),
        Malformed, AdminUnauthenticated, UserNotFound, InvalidReason
    )
)]
pub(super) async fn reactivate_user(
    State(state): State<AppState>,
    user: OnUser,
    Json(body): Json<ReasonBody>,
) -> Result<StatusCode, Problem> {
    state
        .admin
        .reactivate_user(&user.admin, (user.id, &body.reason))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Erases the account `id` as its owner's deletion does: its documents, then the account and
/// everything it had. The audit log keeps its id and the reason, never its address.
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "users",
    operation_id = "deleteUser",
    params(("id" = Uuid, Path, description = "The account's id")),
    request_body = ReasonBody,
    responses(
        (status = NO_CONTENT, description = "Erased"),
        Malformed, AdminUnauthenticated, UserNotFound, InvalidReason
    )
)]
pub(super) async fn delete_user(
    State(state): State<AppState>,
    user: OnUser,
    Json(body): Json<ReasonBody>,
) -> Result<StatusCode, Problem> {
    state
        .admin
        .erase_user(&user.admin, (user.id, &body.reason))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
