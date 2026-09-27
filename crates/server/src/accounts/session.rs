//! The session a request carries: [`resolve`] checks its cookie once, before routing, and puts
//! the outcome in the request's extensions as a [`SessionState`] — and, for a live session, its
//! `AccountId`, which the `api` rate limit counts by. [`CurrentSession`] extracts a live session,
//! or answers the problem of its absence.

use axum::extract::{FromRequestParts, Request, State};
use axum::http::header::SET_COOKIE;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use life_pixel_service::accounts::{Accounts, AccountsError, Authenticated, SecretToken};

use super::cookie;
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

/// What the session cookie of a request turned out to be.
#[derive(Clone, Debug)]
pub enum SessionState {
    /// The request carries no session cookie.
    Absent,
    /// A live session of an active account.
    Live(CurrentSession),
    /// A cookie whose session is unknown, ended or expired: the answer clears it.
    Ended,
    /// A session of a suspended account.
    Suspended,
    /// The stores did not answer.
    Unavailable,
}

/// A live session of an active account, as the handlers that need one extract it: its absence is
/// `auth.unauthenticated`, a suspended account's `auth.account_suspended`.
#[derive(Clone, Debug)]
pub struct CurrentSession {
    /// The cookie's secret: what signing out ends.
    pub token: SecretToken,
    /// The session, and the account signed in.
    pub session: Authenticated,
}

impl SessionState {
    /// The problem a handler that needs a live session answers with, or the session.
    ///
    /// # Errors
    ///
    /// `auth.unauthenticated`, `auth.account_suspended` or `service.unavailable`.
    pub fn live(&self) -> Result<&CurrentSession, Problem> {
        match self {
            Self::Live(current) => Ok(current),
            Self::Absent | Self::Ended => Err(Problem::from(AccountsError::Unauthenticated)),
            Self::Suspended => Err(Problem::from(AccountsError::AccountSuspended)),
            Self::Unavailable => Err(Problem::new(codes::SERVICE_UNAVAILABLE)),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for CurrentSession {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Problem> {
        let Some(state) = parts.extensions.get::<SessionState>() else {
            tracing::error!("a route needs a session, but the session layer is not before it");
            return Err(Problem::new(codes::INTERNAL_ERROR));
        };
        state.live().cloned()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for SessionState {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Problem> {
        let state = parts.extensions.get::<Self>().cloned();
        state.ok_or_else(|| {
            tracing::error!("a route reads the session, but the session layer is not before it");
            Problem::new(codes::INTERNAL_ERROR)
        })
    }
}

/// Middleware of `/api/v1`: checks the session cookie, extends a session seen more than a day
/// ago, and on the way back sends its cookie again — or clears a cookie whose session ended —,
/// unless the handler set the cookie itself.
pub async fn resolve(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let (found, reply) = examine(&state.accounts, request.headers()).await;
    if let SessionState::Live(current) = &found {
        request.extensions_mut().insert(current.session.account.id);
    }
    request.extensions_mut().insert(found);
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if let Some(value) = reply
        && !cookie::is_set_in(headers)
    {
        headers.append(SET_COOKIE, value);
    }
    response
}

/// The state of the request's session, and the cookie its answer sends back: the same one when
/// the session's expiry moved, a cleared one when the session ended.
async fn examine(accounts: &Accounts, headers: &HeaderMap) -> (SessionState, Option<HeaderValue>) {
    let Some(value) = cookie::read(headers) else {
        return (SessionState::Absent, None);
    };
    let found = check(accounts, &value).await;
    let reply = match &found {
        SessionState::Live(current) if current.session.is_due_for_extension => {
            extend(accounts, current).await
        }
        SessionState::Ended => Some(cookie::cleared()),
        _ => None,
    };
    (found, reply)
}

/// The state of the session of the cookie `value`.
async fn check(accounts: &Accounts, value: &str) -> SessionState {
    let Some(token) = SecretToken::parse(value) else {
        return SessionState::Ended;
    };
    match accounts.authenticate(&token).await {
        Ok(session) => SessionState::Live(CurrentSession { token, session }),
        Err(AccountsError::AccountSuspended) => SessionState::Suspended,
        Err(AccountsError::Unavailable) => SessionState::Unavailable,
        Err(_) => SessionState::Ended,
    }
}

/// Moves the expiry of `current` 30 days after now, and returns its cookie to send again; a
/// failure is logged, and the session left as it was.
async fn extend(accounts: &Accounts, current: &CurrentSession) -> Option<HeaderValue> {
    match accounts.extend_session(&current.session.token_hash).await {
        Ok(()) => Some(cookie::set(&current.token)),
        Err(error) => {
            tracing::warn!(%error, "a session's expiry did not move");
            None
        }
    }
}
