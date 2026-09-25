//! The three routers: public, metrics and internal admin, with their middleware.

use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state, map_response_with_state};
use axum::routing::get;
use metrics_exporter_prometheus::PrometheusHandle;
use tower_http::catch_panic::CatchPanicLayer;

use crate::accounts::{self, csrf};
use crate::http::client_version::require_supported;
use crate::http::problem::{
    API_BODY_LIMIT_BYTES, BodyLimit, ensure_problem, not_found, panic_problem,
};
use crate::http::rate_limit::limit_api;
use crate::http::security_headers::{self, SecurityHeaders};
use crate::http::{i18n, request_id, static_app};
use crate::openapi::API_PREFIX;
use crate::routes::{self, health};
use crate::state::AppState;
use crate::telemetry::{observe, render_metrics};

/// The public listener's router: `/healthz`, `/i18n`, `/api/v1`, and the app for the rest. One
/// line per route group; the middleware from the inside out, the request id outermost.
pub fn public_router(state: AppState) -> Router {
    let security = SecurityHeaders::new(state.config.public_url.is_https());
    let proxies = state.config.trusted_proxies.clone();
    Router::new()
        .merge(health::router())
        .nest("/i18n", i18n::router())
        .nest(API_PREFIX, api_router(&state))
        .fallback(static_app::serve)
        .layer(CatchPanicLayer::custom(panic_problem))
        .layer(map_response_with_state(security, security_headers::add))
        .layer(from_fn(observe))
        .layer(from_fn_with_state(proxies, request_id::assign))
        .with_state(state)
}

/// The metrics listener's router: `GET /metrics`.
pub fn metrics_router(handle: PrometheusHandle) -> Router {
    Router::new()
        .route("/metrics", get(render_metrics))
        .with_state(handle)
}

/// The internal admin API's router: `/internal/admin/v1`, which H10 adds. One line per route
/// group.
pub fn admin_router(state: AppState) -> Router {
    let proxies = state.config.trusted_proxies.clone();
    Router::new()
        .fallback(not_found)
        .layer(from_fn(observe))
        .layer(from_fn_with_state(proxies, request_id::assign))
        .with_state(state)
}

/// `/api/v1`: the route groups, then, from the inside out, their rate limits, their CSRF
/// checks, the body limit, the session, the client-version check and the problems of unknown
/// routes and methods.
fn api_router(state: &AppState) -> Router<AppState> {
    let minimums = state.config.min_client_versions.clone();
    let opening = routes::api_session_opening();
    Router::from(
        routes::api_rate_limited()
            .layer(from_fn_with_state(state.clone(), limit_api))
            .merge(routes::api_own_policies())
            .layer(from_fn_with_state(state.clone(), csrf::protect))
            .merge(opening.layer(from_fn_with_state(state.clone(), csrf::check_origin))),
    )
    .fallback(not_found)
    .body_limit(API_BODY_LIMIT_BYTES)
    .layer(from_fn_with_state(state.clone(), accounts::resolve))
    .layer(from_fn_with_state(minimums, require_supported))
    .layer(from_fn(ensure_problem))
}
