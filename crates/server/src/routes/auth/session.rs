//! `GET /auth/session` and `POST /auth/sign-out`: the session a request carries, and its end.

use axum::Json;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use life_pixel_service::accounts::SecretToken;

use super::schema::{Account, Session};
use crate::accounts::metrics::{self, AuthEvent};
use crate::accounts::{CurrentSession, cookie, csrf};
use crate::http::problem::{Problem, ProblemDocument};
use crate::state::AppState;

/// The session the request carries, and its CSRF token.
#[utoipa::path(
    get,
    path = "/auth/session",
    tag = "auth",
    operation_id = "getSession",
    security(("session" = [])),
    responses(
        (status = OK, description = "The session", body = Session),
        (
            status = UNAUTHORIZED,
            description = "`auth.unauthenticated`: no session, or one that ended",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = FORBIDDEN,
            description = "`auth.account_suspended`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn session(
    State(state): State<AppState>,
    current: CurrentSession,
) -> Json<Session> {
    let session = &current.session;
    Json(Session {
        account: Account::from(&session.account),
        csrf_token: csrf::token(&state.config.secrets.session, &session.token_hash),
    })
}

/// Ends the session the request carries, if any, and clears its cookie.
#[utoipa::path(
    post,
    path = "/auth/sign-out",
    tag = "auth",
    operation_id = "signOut",
    security(("session" = [], "csrf" = [])),
    responses(
        (
            status = NO_CONTENT,
            description = "Signed out: `Set-Cookie` clears the session cookie",
            headers(("Set-Cookie" = String, description = "The cleared session cookie"))
        ),
        (
            status = FORBIDDEN,
            description = "`auth.csrf`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
pub(super) async fn sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, Problem> {
    let token = cookie::read(&headers).and_then(|value| SecretToken::parse(&value));
    if let Some(token) = token {
        state.accounts.sign_out(&token).await?;
        metrics::count(AuthEvent::SignOut);
    }
    Ok((StatusCode::NO_CONTENT, [(SET_COOKIE, cookie::cleared())]).into_response())
}
