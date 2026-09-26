//! The session a request carries: [`resolve`] checks its cookie once, and puts the outcome in the
//! request's extensions as a [`SessionState`]; [`require_admin`] lets only a live session
//! through; [`CurrentAdmin`] extracts it. [`cookie`] carries it, [`csrf`] guards what it does.

pub mod cookie;
pub mod csrf;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::header::SET_COOKIE;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::problem::{Problem, codes};
use crate::admins::sessions::SessionToken;
use crate::admins::store::AdminIdentity;
use crate::state::AppState;

/// What the session cookie of a request turned out to be.
#[derive(Clone, Debug)]
pub enum SessionState {
    /// No session cookie.
    Absent,
    /// A live session.
    Live(CurrentAdmin),
    /// A cookie whose session is unknown, ended, or of a disabled admin: the answer clears it.
    Ended,
    /// The database did not answer.
    Unavailable,
}

/// A live session and its admin.
#[derive(Clone, Debug)]
pub struct CurrentAdmin {
    /// The cookie's token.
    pub token: SessionToken,
    /// The admin signed in.
    pub admin: AdminIdentity,
}

impl SessionState {
    /// The live session, or the problem of its absence.
    ///
    /// # Errors
    ///
    /// `admin.unauthenticated`, or `service.unavailable`.
    pub fn live(&self) -> Result<&CurrentAdmin, Problem> {
        match self {
            Self::Live(current) => Ok(current),
            Self::Absent | Self::Ended => Err(Problem::new(codes::ADMIN_UNAUTHENTICATED)),
            Self::Unavailable => Err(Problem::new(codes::SERVICE_UNAVAILABLE)),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for SessionState {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Problem> {
        parts.extensions.get::<Self>().cloned().ok_or_else(|| {
            tracing::error!("a route reads the session, but the session layer is not before it");
            Problem::new(codes::INTERNAL_ERROR)
        })
    }
}

impl<S: Send + Sync> FromRequestParts<S> for CurrentAdmin {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        SessionState::from_request_parts(parts, state)
            .await?
            .live()
            .cloned()
    }
}

/// Middleware of `/api/admin/v1`: checks the session cookie, and clears a cookie whose session
/// ended, unless the handler set the cookie itself.
pub async fn resolve(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let found = match cookie::read(request.headers()) {
        None => SessionState::Absent,
        Some(value) => check(&state, &value).await,
    };
    let is_ended = matches!(found, SessionState::Ended);
    request.extensions_mut().insert(found);
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if is_ended && !cookie::is_set_in(headers) {
        headers.append(SET_COOKIE, cookie::cleared());
    }
    response
}

/// Middleware of the routes only a signed-in admin reaches.
pub async fn require_admin(request: Request, next: Next) -> Response {
    let live = request
        .extensions()
        .get::<SessionState>()
        .map_or(Err(Problem::new(codes::ADMIN_UNAUTHENTICATED)), |state| {
            state.live().map(|_| ())
        });
    match live {
        Ok(()) => next.run(request).await,
        Err(problem) => problem.into_response(),
    }
}

async fn check(state: &AppState, value: &str) -> SessionState {
    let Some(token) = SessionToken::parse(value) else {
        return SessionState::Ended;
    };
    match state.admins.authenticate(&token).await {
        Ok(Some(admin)) => SessionState::Live(CurrentAdmin { token, admin }),
        Ok(None) => SessionState::Ended,
        Err(error) => {
            tracing::error!(%error, "a session could not be checked");
            SessionState::Unavailable
        }
    }
}
