//! The routes, one module per feature, each an `OpenApiRouter` built with `routes!(…)`: the
//! application and the description use the same groups. One line per module and route group.

pub mod health;

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

/// The routes under `/api/v1` limited by the `api` policy: one line per route group.
pub fn api_rate_limited() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
}

/// The routes under `/api/v1` that check a rate-limit policy of their own: one line per route
/// group.
pub fn api_own_policies() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
}
