//! The two routers: the console's — `/healthz`, `/i18n`, `/api/admin/v1` and the built console —
//! and the metrics listener's, with their middleware.

use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state, map_response_with_state};
use axum::routing::get;
use metrics_exporter_prometheus::PrometheusHandle;
use tower_http::catch_panic::CatchPanicLayer;
use utoipa_axum::router::OpenApiRouter;

use crate::admins::routes as auth;
use crate::database;
use crate::http::problem::{
    API_BODY_LIMIT_BYTES, BodyLimit, ensure_problem, not_found, panic_problem,
};
use crate::http::request_id;
use crate::http::security_headers::{self, SecurityHeaders};
use crate::http::session::{self, csrf, require_admin};
use crate::http::static_files::{self, catalogues};
use crate::monitoring;
use crate::relay;
use crate::state::AppState;
use crate::telemetry::{observe, render_metrics};

/// Where the console's API lives.
pub const API_PREFIX: &str = "/api/admin/v1";

/// The console listener's router. One line per route group; the middleware from the inside
/// out, the request id outermost.
pub fn router(state: AppState) -> Router {
    let security = SecurityHeaders::new(state.config.public_url.is_https());
    let proxies = state.config.trusted_proxies.clone();
    Router::new()
        .merge(database::health_router())
        .nest("/i18n", catalogues::router())
        .nest(API_PREFIX, api_router(&state))
        .fallback(static_files::serve)
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

/// The admin server's own routes under `/api/admin/v1` that need a live session, as the
/// OpenAPI description lists them; the relayed routes are the server's.
pub fn signed_in_routes() -> OpenApiRouter<AppState> {
    auth::session_routes().merge(monitoring::routes::router())
}

/// `/api/admin/v1`: the routes of a live session and the relay behind the session check; with
/// sign-out, behind the CSRF check; with sign-in, behind the origin check alone. Then the body
/// limit, the session, and the problems of unknown routes and methods.
fn api_router(state: &AppState) -> Router<AppState> {
    let signed_in = Router::from(signed_in_routes())
        .merge(relay::router())
        .layer(from_fn(require_admin));
    let opening = Router::from(auth::session_opening());
    signed_in
        .merge(auth::session_closing())
        .layer(from_fn_with_state(state.clone(), csrf::protect))
        .merge(opening.layer(from_fn_with_state(state.clone(), csrf::check_origin)))
        .fallback(not_found)
        .body_limit(API_BODY_LIMIT_BYTES)
        .layer(from_fn_with_state(state.clone(), session::resolve))
        .layer(from_fn(ensure_problem))
}
