//! `/api/v1/support-requests` (H9): the signed-in account's support requests — sent as a form
//! with an optional screenshot, listed by cursor, read with their messages, and replied to.
//! Every route needs a session and sits under the CSRF check; sending one checks the
//! `support_create` rate limit and takes a body of up to 6 MiB, the others the `api` limit.

mod creation;
mod form;
mod replies;
mod requests;
pub mod responses;
pub mod schema;

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use life_pixel_service::AccountId;
use life_pixel_service::support::SupportRequestId;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::accounts::CurrentSession;
use crate::http::problem::{BodyLimit, Problem, codes};
use crate::state::AppState;

/// The body limit of a new request: its screenshot and its other parts.
pub const SUPPORT_BODY_LIMIT_BYTES: usize = 6 * 1024 * 1024;

/// The routes under the `api` rate limit and the CSRF check: one line per path.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(requests::list_support_requests))
        .routes(routes!(requests::get_support_request))
        .routes(routes!(replies::reply_to_support_request))
}

/// The routes with a rate limit of their own, under the CSRF check: sending a request, with
/// its larger body limit.
pub fn own_policies() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(creation::create_support_request))
        .body_limit(SUPPORT_BODY_LIMIT_BYTES)
}

/// The account a live session acts for: without one, the routes answer `auth.unauthenticated`,
/// or `auth.account_suspended`.
struct Requester(AccountId);

impl<S: Send + Sync> FromRequestParts<S> for Requester {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        let current = CurrentSession::from_request_parts(parts, state).await?;
        Ok(Self(current.session.account.id))
    }
}

/// A route on one request: the account, then the `{id}` of the path, which answers
/// `request.malformed` when it does not parse.
struct OwnRequest {
    account: AccountId,
    id: SupportRequestId,
}

impl<S: Send + Sync> FromRequestParts<S> for OwnRequest {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        let Requester(account) = Requester::from_request_parts(parts, state).await?;
        let path = Path::<Uuid>::from_request_parts(parts, state).await;
        let Path(id) = path.map_err(|_| Problem::new(codes::REQUEST_MALFORMED))?;
        Ok(Self {
            account,
            id: SupportRequestId::from_uuid(id),
        })
    }
}
