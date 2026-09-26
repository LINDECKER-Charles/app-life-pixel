//! The internal admin API (H10), on `LP_ADMIN_API_ADDR` under `/internal/admin/v1`: only the
//! admin server calls it, with the admin API's secret and the acting admin's identity, and the
//! edge never routes it. Every change is written with its audit entry in one transaction. One
//! line per module.

pub mod auth;
pub mod openapi;
pub mod routes;
pub mod schema;
pub mod storage;

use std::sync::Arc;

use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state};
use life_pixel_service::admin::AdminStores;
use sqlx::PgPool;

use crate::config::HmacKey;
use crate::http::problem::{API_BODY_LIMIT_BYTES, BodyLimit, ensure_problem, not_found};
use crate::state::AppState;
use storage::{
    PostgresAdminSupportStore, PostgresAdminUserStore, PostgresAuditLog, PostgresMetricsSource,
};

/// Where the internal admin API lives.
pub const ADMIN_API_PREFIX: &str = "/internal/admin/v1";

/// `/internal/admin/v1`: the routes, then, from the inside out, the body limit, the admin
/// server's authentication and the problems of unknown routes and methods.
pub fn api_router(state: &AppState) -> Router<AppState> {
    Router::from(routes::routes())
        .fallback(not_found)
        .body_limit(API_BODY_LIMIT_BYTES)
        .layer(from_fn_with_state(state.clone(), auth::authenticate))
        .layer(from_fn(ensure_problem))
}

/// The admin stores over the migrated database of `pool`; `events_secret` finds a user's
/// product events.
#[must_use]
pub fn hosted_stores(pool: &PgPool, events_secret: HmacKey) -> AdminStores {
    AdminStores {
        users: Arc::new(PostgresAdminUserStore::new(pool.clone(), events_secret)),
        support: Arc::new(PostgresAdminSupportStore::new(pool.clone())),
        audit: Arc::new(PostgresAuditLog::new(pool.clone())),
        metrics: Arc::new(PostgresMetricsSource::new(pool.clone())),
    }
}
