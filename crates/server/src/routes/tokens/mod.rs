//! `/api/v1/tokens` (A3): the signed-in account's personal access tokens — listed, created with
//! their secret shown once, and revoked. Every route needs a session, sits under the CSRF check
//! and the `api` rate limit.

mod handlers;
pub mod responses;
pub mod schema;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::state::AppState;

/// The routes under the `api` rate limit and the CSRF check: one line per path.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::list_tokens, handlers::create_token))
        .routes(routes!(handlers::revoke_token))
}
