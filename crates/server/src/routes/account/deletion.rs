//! `DELETE /account`: the signed-in account deleted at its owner's request, its password asked
//! again — its documents first, then the account and, with it, everything else it had.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use life_pixel_service::accounts::AccountDeletion;

use super::schema::AccountDeletionRequest;
use crate::accounts::{CurrentSession, cookie};
use crate::http::problem::{Problem, ProblemDocument};
use crate::routes::library::responses::{Malformed, RateLimited, Unauthenticated};
use crate::state::AppState;

/// Deletes the signed-in account, its library and its sessions, and clears the session cookie.
#[utoipa::path(
    delete,
    path = "/account",
    tag = "account",
    operation_id = "deleteAccount",
    security(("session" = [], "csrf" = [])),
    request_body = AccountDeletionRequest,
    responses(
        (
            status = NO_CONTENT,
            description = "The account is deleted: `Set-Cookie` clears the session cookie",
            headers(("Set-Cookie" = String, description = "The cleared session cookie"))
        ),
        Malformed, Unauthenticated,
        (
            status = FORBIDDEN,
            description = "`auth.current_password`, `auth.account_suspended`, `auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        RateLimited
    )
)]
pub(super) async fn delete_account(
    State(state): State<AppState>,
    current: CurrentSession,
    Json(request): Json<AccountDeletionRequest>,
) -> Result<Response, Problem> {
    let deletion = AccountDeletion {
        account: current.session.account.id,
        password: request.password,
    };
    state
        .accounts
        .delete_with_library(&state.library, deletion)
        .await?;
    Ok((StatusCode::NO_CONTENT, [(SET_COOKIE, cookie::cleared())]).into_response())
}
