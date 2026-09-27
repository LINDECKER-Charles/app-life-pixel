//! The variables of the local stack, read from the repository's `.env`, and what can go wrong
//! setting a test up.

use std::path::Path;
use std::sync::Once;

use thiserror::Error;

use crate::config::ConfigError;
use crate::database::DatabaseError;
use crate::storage::ObjectStoreSetupError;

/// The repository's development environment file: the one of this crate's checkout, never one
/// found by walking up from the working directory — a worktree lies inside another checkout.
const REPOSITORY_ENV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.env");

/// Why a test could not get its share of the stack.
#[derive(Debug, Error)]
pub enum TestSetupError {
    /// A variable is missing: `.env` is not copied from `.env.example`.
    #[error("{0} is missing: copy .env.example to .env")]
    Missing(&'static str),
    /// A variable does not parse.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The database server refused a statement.
    #[error("the test database server: {0}")]
    Database(#[from] sqlx::Error),
    /// The test database cannot be migrated.
    #[error(transparent)]
    Migrate(#[from] DatabaseError),
    /// A database URL names another database than a test database.
    #[error("{0} is not a test database: only lp_test_ databases are dropped")]
    NotATestDatabase(String),
    /// The object storage cannot be set up.
    #[error(transparent)]
    ObjectStoreSetup(#[from] ObjectStoreSetupError),
    /// The object storage failed.
    #[error("the test object storage: {0}")]
    Objects(#[from] object_store::Error),
    /// Mailpit's API failed.
    #[error("Mailpit's API: {0}")]
    Mail(#[from] reqwest::Error),
    /// No message arrived in time.
    #[error("no message for {0} arrived in time")]
    NoMessage(String),
}

/// Loads the repository's `.env` once, without overriding a variable already set.
pub fn load_test_env() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let path = Path::new(REPOSITORY_ENV_FILE);
        if path.exists() && dotenvy::from_path(path).is_err() {
            tracing::warn!("the repository's .env cannot be read");
        }
    });
}

/// The value of `variable`, after the repository's `.env` is loaded.
pub(super) fn variable(variable: &'static str) -> Result<String, TestSetupError> {
    load_test_env();
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(TestSetupError::Missing(variable))
}
