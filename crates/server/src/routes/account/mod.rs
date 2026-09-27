//! `/api/v1/account` (H6): the signed-in account itself — its usage against the quota, its
//! language, the export of its data, and its deletion. Every route needs a session, and sits
//! under the `api` rate limit and the CSRF check.

mod deletion;
mod export;
mod profile;
pub mod schema;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::state::AppState;

/// The routes under the `api` rate limit and the CSRF check: one line per path.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            profile::get_account,
            profile::update_account,
            deletion::delete_account
        ))
        .routes(routes!(export::export_account))
}
