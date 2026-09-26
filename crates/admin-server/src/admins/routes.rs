//! `/auth`: signing in, reading the session, signing out.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::password::Password;
use super::sessions::SessionToken;
use super::store::AdminIdentity;
use super::{Credentials, SignInError};
use crate::http::client_address::ClientAddress;
use crate::http::problem::{Problem, ProblemDocument, codes};
use crate::http::session::{CurrentAdmin, SessionState, cookie, csrf};
use crate::state::AppState;

/// What signing in sends.
#[derive(Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    /// The admin's address.
    pub email: String,
    /// The password.
    pub password: String,
    /// The current code of the admin's authenticator: six digits.
    pub code: String,
}

/// The admin signed in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleAdmin {
    /// The admin's id.
    pub id: Uuid,
    /// The admin's address.
    pub email: String,
}

/// A live session: who is signed in, the token every unsafe request sends in `X-CSRF-Token`,
/// the environment this console administers and those its monitoring shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleSession {
    /// The admin signed in.
    pub admin: ConsoleAdmin,
    /// The session's CSRF token.
    pub csrf_token: String,
    /// `LPA_ENVIRONMENT`: `local`, `staging` or `production`.
    pub environment: String,
    /// `LPA_ENVIRONMENTS`: the environments the monitoring routes take as `env`.
    pub monitored_environments: Vec<String>,
}

/// The routes that open a session: only the origin is checked.
pub fn session_opening() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(sign_in))
}

/// The route that ends a session, live or not: CSRF-checked, reachable without a live session.
pub fn session_closing() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(sign_out))
}

/// The routes of a live session.
pub fn session_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(session))
}

/// Opens a session: `Set-Cookie: __Host-lpa_session`, and the session. A wrong address,
/// password or code is `admin.invalid_credentials`, whichever it is.
#[utoipa::path(
    post,
    path = "/auth/sign-in",
    tag = "auth",
    operation_id = "signIn",
    request_body = SignInRequest,
    responses(
        (status = OK, description = "Signed in", body = ConsoleSession),
        (status = BAD_REQUEST, description = "`request.malformed`", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = UNAUTHORIZED, description = "`admin.invalid_credentials`",
            body = ProblemDocument, content_type = "application/problem+json"),
        (status = FORBIDDEN, description = "`admin.csrf`: another origin", body = ProblemDocument,
            content_type = "application/problem+json"),
        (status = TOO_MANY_REQUESTS, description = "`rate_limit.exceeded`: 5 attempts a minute \
            per address and per email", body = ProblemDocument,
            content_type = "application/problem+json"),
    )
)]
async fn sign_in(
    State(state): State<AppState>,
    ClientAddress(address): ClientAddress,
    Json(body): Json<SignInRequest>,
) -> Result<Response, Problem> {
    state.sign_in_limits.check(address, &body.email)?;
    let credentials = Credentials {
        email: body.email,
        password: Password::sent(body.password),
        code: body.code,
    };
    let signed_in = state
        .admins
        .sign_in(&credentials)
        .await
        .map_err(|error| match error {
            SignInError::InvalidCredentials => Problem::new(codes::ADMIN_INVALID_CREDENTIALS),
            SignInError::Unavailable(error) => {
                tracing::error!(%error, "a sign-in could not be checked");
                Problem::new(codes::SERVICE_UNAVAILABLE)
            }
        })?;
    tracing::info!(admin = %signed_in.admin.id, "an admin signed in");
    let body = session_body(&state, &signed_in.token, &signed_in.admin);
    let mut response = Json(body).into_response();
    let cookie = cookie::set(&signed_in.token);
    response.headers_mut().append(SET_COOKIE, cookie);
    Ok(response)
}

/// The live session.
#[utoipa::path(
    get,
    path = "/auth/session",
    tag = "auth",
    operation_id = "getSession",
    responses(
        (status = OK, description = "The live session", body = ConsoleSession),
        (status = UNAUTHORIZED, description = "`admin.unauthenticated`: no live session",
            body = ProblemDocument, content_type = "application/problem+json"),
    ),
    security(("adminSession" = []))
)]
async fn session(State(state): State<AppState>, current: CurrentAdmin) -> Json<ConsoleSession> {
    Json(session_body(&state, &current.token, &current.admin))
}

/// Ends the session, if any, and clears its cookie.
#[utoipa::path(
    post,
    path = "/auth/sign-out",
    tag = "auth",
    operation_id = "signOut",
    responses(
        (status = NO_CONTENT, description = "Signed out"),
        (status = FORBIDDEN, description = "`admin.csrf`", body = ProblemDocument,
            content_type = "application/problem+json"),
    ),
    security(("adminSession" = [], "adminCsrf" = []))
)]
async fn sign_out(State(state): State<AppState>, session: SessionState) -> Response {
    if let SessionState::Live(current) = session
        && let Err(error) = state.admins.sign_out(&current.token).await
    {
        tracing::error!(%error, "a session could not be ended");
        return Problem::new(codes::SERVICE_UNAVAILABLE).into_response();
    }
    (StatusCode::NO_CONTENT, [(SET_COOKIE, cookie::cleared())]).into_response()
}

fn session_body(state: &AppState, token: &SessionToken, admin: &AdminIdentity) -> ConsoleSession {
    let config = &state.config;
    ConsoleSession {
        admin: ConsoleAdmin {
            id: admin.id,
            email: admin.email.clone(),
        },
        csrf_token: csrf::token(&config.session_secret, &token.hash()),
        environment: config.environment.as_str().to_owned(),
        monitored_environments: config.monitoring.environments.names().to_vec(),
    }
}
