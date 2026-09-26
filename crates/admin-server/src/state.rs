//! The state every handler shares, built once at start. One line per field.

use std::sync::Arc;

use sqlx::PgPool;
use thiserror::Error;

use crate::admins::Admins;
use crate::admins::clock::Clock;
use crate::admins::rate_limit::SignInLimits;
use crate::admins::secret_box::SecretBox;
use crate::admins::store::AdminStore;
use crate::config::Config;
use crate::http::static_files::StaticConsole;
use crate::http::static_files::catalogues::{CatalogueError, Catalogues};
use crate::monitoring::{Monitoring, source};
use crate::relay;

/// Why the state could not be built.
#[derive(Debug, Error)]
pub enum StartError {
    /// The catalogues of `LPA_I18N_DIR` could not be read.
    #[error("LPA_I18N_DIR: {0}")]
    Catalogues(#[from] CatalogueError),
    /// The built console of `LPA_APP_DIR` could not be read.
    #[error("LPA_APP_DIR: {0}")]
    Console(#[from] std::io::Error),
    /// An HTTP client could not start.
    #[error("an HTTP client cannot start: {0}")]
    Client(#[from] reqwest::Error),
}

/// The shared state: cheap to clone, one field per line.
#[derive(Clone)]
pub struct AppState {
    /// The configuration.
    pub config: Arc<Config>,
    /// The admin server's database, for `/healthz`.
    pub pool: PgPool,
    /// The admin accounts and their sessions.
    pub admins: Admins,
    /// The sign-in rate limits.
    pub sign_in_limits: Arc<SignInLimits>,
    /// The client of the internal admin API.
    pub relay_client: reqwest::Client,
    /// The monitoring sources.
    pub monitoring: Monitoring,
    /// The built console.
    pub static_console: Arc<StaticConsole>,
    /// The catalogues of `/i18n`.
    pub catalogues: Arc<Catalogues>,
}

impl AppState {
    /// Reads the catalogues and the built console, and gathers the rest around `pool` and
    /// `clock`.
    ///
    /// # Errors
    ///
    /// When the catalogues or the console cannot be read, or an HTTP client cannot start.
    pub fn new(config: Config, pool: PgPool, clock: Arc<dyn Clock>) -> Result<Self, StartError> {
        let catalogues = Catalogues::load(&config.i18n_dir)?;
        let static_console = StaticConsole::load(&config.app_dir)?;
        let secrets = SecretBox::new(&config.totp_key);
        let admins = Admins::new(AdminStore::new(pool.clone()), secrets, clock);
        let relay_client = relay::client()?;
        let monitoring = Monitoring::new(config.monitoring.clone(), source::client()?);
        Ok(Self {
            config: Arc::new(config),
            pool,
            admins,
            sign_in_limits: Arc::new(SignInLimits::new()),
            relay_client,
            monitoring,
            static_console: Arc::new(static_console),
            catalogues: Arc::new(catalogues),
        })
    }
}
