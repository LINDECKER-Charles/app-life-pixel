//! The variables of the local stack, read from the repository's `.env`, what can go wrong
//! setting a test up, and cleaning up from `Drop`, which cannot await: the cleanup runs to its
//! end on a thread of its own, with a runtime and connections of its own.

use std::future::Future;
use std::path::Path;
use std::sync::Once;

use thiserror::Error;

use crate::config::ConfigError;
use crate::database::DatabaseError;

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
    #[error("{0} is not a test database: only lpa_test_ databases are dropped")]
    NotATestDatabase(String),
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

/// Runs `cleanup` to its end on a new thread; a failure is logged.
pub(super) fn run_to_end<F>(what: &'static str, cleanup: impl FnOnce() -> F + Send + 'static)
where
    F: Future<Output = Result<(), TestSetupError>>,
{
    let thread = std::thread::spawn(move || block_on(cleanup()));
    let error = match thread.join() {
        Ok(Ok(())) => return,
        Ok(Err(error)) => error.to_string(),
        Err(_) => "the cleanup panicked".to_owned(),
    };
    tracing::warn!(%error, what, "a test resource was not cleaned up");
}

/// Runs `cleanup` on a runtime of its own.
fn block_on<F>(cleanup: F) -> std::io::Result<()>
where
    F: Future<Output = Result<(), TestSetupError>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(cleanup).map_err(std::io::Error::other)
}
