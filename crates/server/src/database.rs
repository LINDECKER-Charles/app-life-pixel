//! The database, until H4 brings its pool and migrations: its address is checked for a
//! connection, which is all `/healthz` and `migrate` can know without a client.

use std::io;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::config::DatabaseConfig;
use crate::readiness::Readiness;

/// How long the database has to accept a connection.
pub const DATABASE_TIMEOUT: Duration = Duration::from_secs(1);

/// Why the migrations could not run.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// The database's address refused the connection, or did not answer in time.
    #[error("the database at {address} does not answer: {source}")]
    Unreachable {
        /// `host:port` of `LP_DATABASE_URL`, without its credentials.
        address: String,
        /// The failure.
        source: io::Error,
    },
}

/// The readiness of the database: its address accepts a TCP connection within a second.
#[derive(Clone, Debug)]
pub struct DatabaseProbe {
    address: String,
}

impl DatabaseProbe {
    /// The probe of the database `config` names.
    #[must_use]
    pub fn new(config: &DatabaseConfig) -> Self {
        Self {
            address: config.address.clone(),
        }
    }

    /// Connects to the database's address, then closes the connection.
    ///
    /// # Errors
    ///
    /// When the connection is refused, or takes more than [`DATABASE_TIMEOUT`].
    pub async fn connect(&self) -> Result<(), DatabaseError> {
        let unreachable = |source| DatabaseError::Unreachable {
            address: self.address.clone(),
            source,
        };
        let connection = tokio::time::timeout(DATABASE_TIMEOUT, TcpStream::connect(&self.address))
            .await
            .map_err(|elapsed| unreachable(io::Error::new(io::ErrorKind::TimedOut, elapsed)))?;
        connection.map(drop).map_err(unreachable)
    }
}

#[async_trait]
impl Readiness for DatabaseProbe {
    async fn is_ready(&self) -> bool {
        self.connect().await.is_ok()
    }
}

/// Runs the migrations. H3 has none yet: it checks that the database answers, so that `serve`
/// and `migrate` fail as they will once H4 adds them.
///
/// # Errors
///
/// When the database does not answer.
pub async fn migrate(config: &DatabaseConfig) -> Result<(), DatabaseError> {
    DatabaseProbe::new(config).connect().await?;
    tracing::info!("no migration to run yet");
    Ok(())
}
