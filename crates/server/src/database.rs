//! The database: its connection pool, its migrations, and its readiness for `/healthz`.

use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use thiserror::Error;

use crate::config::DatabaseConfig;
use crate::readiness::Readiness;

/// How long the database has to answer `/healthz`.
pub const DATABASE_TIMEOUT: Duration = Duration::from_secs(1);
/// How long a query waits for a free connection before it fails.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
/// The query that checks the database answers.
const PING: &str = "select 1";

/// The migrations of `crates/server/migrations/`, embedded at build time.
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Why the database could not be used.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// `LP_DATABASE_URL` is not a URL sqlx accepts.
    #[error("LP_DATABASE_URL is invalid: {0}")]
    Url(sqlx::Error),
    /// The database refused the connection, or did not answer in time.
    #[error("the database at {address} does not answer: {source}")]
    Unreachable {
        /// `host:port` of `LP_DATABASE_URL`, without its credentials.
        address: String,
        /// The failure.
        source: sqlx::Error,
    },
    /// A migration failed.
    #[error("the migrations failed: {0}")]
    Migrate(#[from] MigrateError),
}

/// The pool of `config`, once a first connection has succeeded.
///
/// # Errors
///
/// When the URL is invalid, or the database does not answer.
pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, DatabaseError> {
    let options = PgConnectOptions::from_str(config.url.expose()).map_err(DatabaseError::Url)?;
    pool_options(config.max_connections.get())
        .connect_with(options)
        .await
        .map_err(|source| DatabaseError::Unreachable {
            address: config.address.clone(),
            source,
        })
}

/// The pool settings, `max_connections` at most.
#[must_use]
pub fn pool_options(max_connections: u32) -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(ACQUIRE_TIMEOUT)
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

/// The readiness of the database: the pool runs a query.
#[derive(Clone, Debug)]
pub struct DatabaseReadiness {
    pool: PgPool,
}

impl DatabaseReadiness {
    /// The readiness of the database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Readiness for DatabaseReadiness {
    async fn is_ready(&self) -> bool {
        let ping = sqlx::query(PING).execute(&self.pool);
        matches!(
            tokio::time::timeout(DATABASE_TIMEOUT, ping).await,
            Ok(Ok(_))
        )
    }
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
