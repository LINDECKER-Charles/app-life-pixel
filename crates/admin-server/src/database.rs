//! The admin server's own database, `life_pixel_admin`: its connection pool, its migrations, and
//! `GET /healthz`, which answers when it does.

use std::str::FromStr;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use thiserror::Error;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::config::SecretString;
use crate::http::problem::{Problem, ProblemDocument, codes};
use crate::state::AppState;

/// How long the database has to answer `/healthz`.
pub const DATABASE_TIMEOUT: Duration = Duration::from_secs(1);
/// The connections the pool opens at most: a handful of admins sign in.
pub const MAX_CONNECTIONS: u32 = 5;
/// How long a query waits for a free connection before it fails.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
/// The query that checks the database answers.
const PING: &str = "select 1";

/// The migrations of `crates/admin-server/migrations/`, embedded at build time.
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Why the database could not be used.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// `LPA_DATABASE_URL` is not a URL sqlx accepts.
    #[error("LPA_DATABASE_URL is invalid: {0}")]
    Url(sqlx::Error),
    /// The database refused the connection, or did not answer in time.
    #[error("the database at {address} does not answer: {source}")]
    Unreachable {
        /// `host:port` of `LPA_DATABASE_URL`, without its credentials.
        address: String,
        /// The failure.
        source: sqlx::Error,
    },
    /// A migration failed.
    #[error("the migrations failed: {0}")]
    Migrate(#[from] MigrateError),
}

/// The pool of `url`, once a first connection has succeeded.
///
/// # Errors
///
/// When the URL is invalid, or the database does not answer.
pub async fn connect(url: &SecretString) -> Result<PgPool, DatabaseError> {
    let options = PgConnectOptions::from_str(url.expose()).map_err(DatabaseError::Url)?;
    let address = format!("{}:{}", options.get_host(), options.get_port());
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(|source| DatabaseError::Unreachable { address, source })
}

/// Runs the migrations not yet applied to the database of `pool`.
///
/// # Errors
///
/// When a migration fails, or one already applied has changed.
pub async fn migrate(pool: &PgPool) -> Result<(), DatabaseError> {
    MIGRATOR.run(pool).await?;
    tracing::info!(
        migrations = MIGRATOR.iter().count(),
        "the database is migrated"
    );
    Ok(())
}

/// The admin server's health.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct Health {
    /// Always `ok`: an admin server that cannot answer sends a problem.
    pub status: HealthStatus,
}

/// The status of a healthy admin server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    /// The database answers.
    Ok,
}

/// `/healthz`.
pub fn health_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(healthz))
}

/// Whether the admin server's database answers within a second.
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    operation_id = "healthz",
    responses(
        (status = OK, description = "The database answers", body = Health),
        (
            status = SERVICE_UNAVAILABLE,
            description = "The database does not answer: `service.unavailable`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
async fn healthz(State(state): State<AppState>) -> Result<Json<Health>, Problem> {
    let ping = sqlx::query(PING).execute(&state.pool);
    if !matches!(
        tokio::time::timeout(DATABASE_TIMEOUT, ping).await,
        Ok(Ok(_))
    ) {
        return Err(Problem::new(codes::SERVICE_UNAVAILABLE));
    }
    Ok(Json(Health {
        status: HealthStatus::Ok,
    }))
}

#[cfg(test)]
mod tests {
    use time::PrimitiveDateTime;
    use time::macros::format_description;

    use super::*;

    #[test]
    fn every_migration_is_named_after_a_utc_timestamp_and_a_topic() {
        let timestamp = format_description!("[year][month][day][hour][minute][second]");
        for migration in MIGRATOR.iter() {
            let version = migration.version.to_string();
            let parsed = PrimitiveDateTime::parse(&version, &timestamp);
            assert!(parsed.is_ok(), "{version} is not a UTC timestamp");
            assert!(!migration.description.is_empty(), "{version} has no topic");
        }
    }
}
