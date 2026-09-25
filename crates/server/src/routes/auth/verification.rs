//! `POST /auth/verify-email` and `POST /auth/verify-email/resend`: the emailed link that verifies
//! an address, and a new one.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use life_pixel_service::AccountId;

use super::schema::VerifyEmailRequest;
use crate::accounts::CurrentSession;
use crate::http::problem::{Problem, ProblemDocument};
use crate::http::rate_limit::{Policy, RateKey};
use crate::state::AppState;

/// Verifies the address of the account an emailed link was sent to, and spends its token.
#[utoipa::path(
    post,
    path = "/auth/verify-email",
    tag = "auth",
    operation_id = "verifyEmail",
    request_body = VerifyEmailRequest,
    responses(
        (status = NO_CONTENT, description = "The address is verified"),
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
    )
)]
pub(super) async fn verify_email(
    State(state): State<AppState>,
    Json(request): Json<VerifyEmailRequest>,
) -> Result<StatusCode, Problem> {
    state.accounts.verify_email(&request.token).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Sends the signed-in account's verification email again, with a new link; nothing once the
/// address is verified.
#[utoipa::path(
    post,
    path = "/auth/verify-email/resend",
    tag = "auth",
    operation_id = "resendVerificationEmail",
    security(("session" = [], "csrf" = [])),
    responses(
        (status = ACCEPTED, description = "The email is on its way, if the address is unverified"),
        (
            status = UNAUTHORIZED,
            description = "`auth.unauthenticated`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = FORBIDDEN,
            description = "`auth.account_suspended`, `auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "`rate_limit.exceeded`: 3 an hour per account",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn resend_verification(
    State(state): State<AppState>,
    current: CurrentSession,
) -> Result<StatusCode, Problem> {
    let account: AccountId = current.session.account.id;
    let key = RateKey::Account(account);
    state.rate_limits.check(Policy::VerificationResend, &key)?;
    state.accounts.resend_verification(account).await?;
    Ok(StatusCode::ACCEPTED)
}
