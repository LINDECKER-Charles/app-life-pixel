//! The routes, one module per feature, each an `OpenApiRouter` built with `routes!(…)`: the
//! application and the description use the same groups. One line per module and route group.

pub mod account;
pub mod auth;
pub mod events;
pub mod exports;
pub mod health;
pub mod library;
pub mod support;
pub mod tokens;

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

/// The routes under `/api/v1` limited by the `api` policy: one line per route group.
pub fn api_rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(auth::rate_limited())
        .merge(library::rate_limited())
        .merge(account::rate_limited())
        .merge(support::rate_limited())
        .merge(tokens::rate_limited())
        .merge(exports::rate_limited())
}

/// The routes under `/api/v1` that check a rate-limit policy of their own: one line per route
/// group.
pub fn api_own_policies() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(auth::own_policies())
        .merge(events::own_policies())
        .merge(support::own_policies())
}

/// The routes under `/api/v1` that open a session: they check a rate-limit policy of their own,
/// and the request's origin instead of a CSRF token. One line per route group.
pub fn api_session_opening() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().merge(auth::opening())
}
