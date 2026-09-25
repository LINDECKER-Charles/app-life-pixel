//! `POST /auth/password-reset`, `POST /auth/password-reset/confirm` and `PUT /auth/password`:
//! a forgotten password reset through an emailed link, and a known one changed.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use life_pixel_service::accounts::{PasswordChange, PasswordReset};

use super::schema::{PasswordChangeRequest, PasswordResetConfirmation, PasswordResetRequest};
use crate::accounts::CurrentSession;
use crate::accounts::metrics::{self, AuthEvent};
use crate::http::client_address::ClientAddress;
use crate::http::problem::{Problem, ProblemDocument};
use crate::http::rate_limit::{Policy, RateKey};
use crate::state::AppState;

/// Emails a link setting a new password, when an active account has the address. The answer is
/// the same, and as quick, whether it has or not.
#[utoipa::path(
    post,
    path = "/auth/password-reset",
    tag = "auth",
    operation_id = "requestPasswordReset",
    request_body = PasswordResetRequest,
    responses(
        (status = ACCEPTED, description = "A link is on its way, if an account has the address"),
        (
            status = FORBIDDEN,
            description = "`auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "`rate_limit.exceeded`: 5 an hour per address and per email",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn request_reset(
    State(state): State<AppState>,
    ClientAddress(address): ClientAddress,
    Json(request): Json<PasswordResetRequest>,
) -> Result<StatusCode, Problem> {
    let limits = &state.rate_limits;
    limits.check(Policy::PasswordReset, &RateKey::Address(address))?;
    limits.check(Policy::PasswordReset, &RateKey::email(&request.email))?;
    state.accounts.request_password_reset(&request.email);
    Ok(StatusCode::ACCEPTED)
}

/// Sets a new password with the token of an emailed link, and ends every session of the account.
#[utoipa::path(
    post,
    path = "/auth/password-reset/confirm",
    tag = "auth",
    operation_id = "confirmPasswordReset",
    request_body = PasswordResetConfirmation,
    responses(
        (status = NO_CONTENT, description = "The password is set; every session has ended"),
        (
            status = BAD_REQUEST,
            description = "`auth.token_invalid`: unknown, used or expired",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = FORBIDDEN,
            description = "`auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = UNPROCESSABLE_ENTITY,
            description = "`auth.password_length` (`min`, `max`); the token still works",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn confirm_reset(
    State(state): State<AppState>,
    Json(request): Json<PasswordResetConfirmation>,
) -> Result<StatusCode, Problem> {
    let reset = PasswordReset {
        token: request.token,
        password: request.password,
    };
    state.accounts.confirm_password_reset(reset).await?;
    metrics::count(AuthEvent::PasswordReset);
    Ok(StatusCode::NO_CONTENT)
}

/// Replaces the signed-in account's password, and ends its other sessions.
#[utoipa::path(
    put,
    path = "/auth/password",
    tag = "auth",
    operation_id = "changePassword",
    security(("session" = [], "csrf" = [])),
    request_body = PasswordChangeRequest,
    responses(
        (status = NO_CONTENT, description = "The password is changed; the other sessions ended"),
        (
            status = UNAUTHORIZED,
            description = "`auth.unauthenticated`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = FORBIDDEN,
            description = "`auth.current_password`, `auth.account_suspended`, `auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = UNPROCESSABLE_ENTITY,
            description = "`auth.password_length` (`min`, `max`)",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "`rate_limit.exceeded`: 5 a minute per account, as signing in",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn change_password(
    State(state): State<AppState>,
    current: CurrentSession,
    Json(request): Json<PasswordChangeRequest>,
) -> Result<StatusCode, Problem> {
    let email = RateKey::email(&current.session.account.email);
    state.rate_limits.check(Policy::SignIn, &email)?;
    let change = PasswordChange {
        current_password: request.current_password,
        new_password: request.new_password,
    };
    state
        .accounts
        .change_password(&current.session, change)
        .await?;
    metrics::count(AuthEvent::PasswordChanged);
    Ok(StatusCode::NO_CONTENT)
}
