//! `/api/v1/auth`: signing up and in, the session, signing out, verifying the address, resetting
//! and changing the password (H5), in three groups by their rate limits and CSRF checks.

mod opening;
mod passwords;
pub mod schema;
mod session;
mod verification;

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use life_pixel_service::accounts::SignedIn;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use self::schema::{Account, Session};
use crate::accounts::{cookie, csrf};
use crate::state::AppState;

/// The routes that open a session, with rate limits of their own and an origin check.
pub fn opening() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(opening::sign_up))
        .routes(routes!(opening::sign_in))
}

/// The routes under the `api` rate limit and the CSRF check.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(session::session))
        .routes(routes!(session::sign_out))
        .routes(routes!(verification::verify_email))
        .routes(routes!(passwords::confirm_reset))
}

/// The routes with rate limits of their own, under the CSRF check.
pub fn own_policies() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(verification::resend_verification))
        .routes(routes!(passwords::request_reset))
        .routes(routes!(passwords::change_password))
}

/// The answer of a session just opened: `status`, its cookie, and the session.
fn session_answer(state: &AppState, status: StatusCode, signed_in: &SignedIn) -> Response {
    let key = &state.config.secrets.session;
    let session = Session {
        account: Account::from(&signed_in.account),
        csrf_token: csrf::token(key, &signed_in.token.hash()),
    };
    let cookie = [(SET_COOKIE, cookie::set(&signed_in.token))];
    (status, cookie, Json(session)).into_response()
}
