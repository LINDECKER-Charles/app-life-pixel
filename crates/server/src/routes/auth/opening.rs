//! `POST /auth/sign-up` and `POST /auth/sign-in`: the routes that open a session. They check
//! the request's origin, not a CSRF token: there is no session yet.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use life_pixel_service::accounts::{AccountsError, Credentials, SignUp};

use super::schema::{Session, SignInRequest, SignUpRequest};
use super::session_answer;
use crate::accounts::metrics::{self, AuthEvent};
use crate::http::client_address::ClientAddress;
use crate::http::problem::{Problem, ProblemDocument};
use crate::http::rate_limit::{Policy, RateKey};
use crate::state::AppState;

/// Creates an account, signs it in, and sends the email that verifies its address.
#[utoipa::path(
    post,
    path = "/auth/sign-up",
    tag = "auth",
    operation_id = "signUp",
    request_body = SignUpRequest,
    responses(
        (
            status = CREATED,
            description = "The account, signed in: `Set-Cookie` holds its session",
            body = Session,
            headers(("Set-Cookie" = String, description = "The session cookie"))
        ),
        (
            status = FORBIDDEN,
            description = "`auth.csrf`: from another origin",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = CONFLICT,
            description = "`auth.email_taken`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = UNPROCESSABLE_ENTITY,
            description = "`auth.email_invalid`, `auth.password_length` (`min`, `max`), \
                           `account.language` (`available`)",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "`rate_limit.exceeded`: 5 an hour per address",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn sign_up(
    State(state): State<AppState>,
    ClientAddress(address): ClientAddress,
    Json(request): Json<SignUpRequest>,
) -> Result<Response, Problem> {
    state
        .rate_limits
        .check(Policy::SignUp, &RateKey::Address(address))?;
    let sign_up = SignUp {
        email: request.email,
        password: request.password,
        language: request.language,
    };
    let signed_in = state.accounts.sign_up(sign_up).await?;
    metrics::count(AuthEvent::SignUp);
    Ok(session_answer(&state, StatusCode::CREATED, &signed_in))
}

/// Opens a session with an address and its password.
#[utoipa::path(
    post,
    path = "/auth/sign-in",
    tag = "auth",
    operation_id = "signIn",
    request_body = SignInRequest,
    responses(
        (
            status = OK,
            description = "Signed in: `Set-Cookie` holds the session",
            body = Session,
            headers(("Set-Cookie" = String, description = "The session cookie"))
        ),
        (
            status = UNAUTHORIZED,
            description = "`auth.invalid_credentials`: an unknown address or a wrong password",
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
            description = "`rate_limit.exceeded`: 10 a minute per address, 5 per address signed \
                           in to",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn sign_in(
    State(state): State<AppState>,
    ClientAddress(address): ClientAddress,
    Json(request): Json<SignInRequest>,
) -> Result<Response, Problem> {
    let limits = &state.rate_limits;
    limits.check(Policy::SignIn, &RateKey::Address(address))?;
    limits.check(Policy::SignIn, &RateKey::email(&request.email))?;
    let credentials = Credentials {
        email: request.email,
        password: request.password,
    };
    let signed_in = state.accounts.sign_in(credentials).await;
    let signed_in = signed_in.inspect_err(|error| {
        if matches!(
            error,
            AccountsError::InvalidCredentials | AccountsError::AccountSuspended
        ) {
            metrics::count(AuthEvent::SignInFailed);
        }
    })?;
    metrics::count(AuthEvent::SignIn);
    Ok(session_answer(&state, StatusCode::OK, &signed_in))
}
