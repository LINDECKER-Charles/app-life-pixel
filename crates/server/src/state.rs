//! The state every handler shares, built once at start. One line per field.

use std::sync::Arc;

use life_pixel_service::accounts::{Accounts, AccountsPorts};
use life_pixel_service::library::{Library, LibraryPorts};
use life_pixel_service::ports::{EventSink, LibraryStore};
use life_pixel_service::support::{Support, SupportPorts, SupportStores};
use thiserror::Error;

use crate::accounts;
use crate::config::Config;
use crate::http::i18n::{CatalogueError, Catalogues};
use crate::http::rate_limit::RateLimits;
use crate::http::static_app::StaticApp;
use crate::readiness::Readiness;

/// Why the state could not be built.
#[derive(Debug, Error)]
pub enum StartError {
    /// The catalogues of `LP_I18N_DIR` could not be read.
    #[error("LP_I18N_DIR: {0}")]
    Catalogues(#[from] CatalogueError),
    /// The built app of `LP_APP_DIR` could not be read.
    #[error("LP_APP_DIR: {0}")]
    App(#[from] std::io::Error),
}

/// What the state is built on, made by `serve` from the configuration or by a test: one field
/// per line.
pub struct Backends {
    /// Whether the database answers.
    pub readiness: Arc<dyn Readiness>,
    /// The hosted library.
    pub library_store: Arc<dyn LibraryStore>,
    /// The accounts' stores, mailer, clock, ids and product events.
    pub accounts: AccountsPorts,
    /// Where the app's own product events go (H13).
    pub events: Arc<dyn EventSink>,
    /// Where support requests and their screenshots are kept (H9).
    pub support: SupportStores,
}

/// The shared state: cheap to clone, one field per line.
#[derive(Clone)]
pub struct AppState {
    /// The configuration.
    pub config: Arc<Config>,
    /// Whether the database answers.
    pub readiness: Arc<dyn Readiness>,
    /// The hosted library.
    pub library_store: Arc<dyn LibraryStore>,
    /// The library's use cases, over the hosted library.
    pub library: Library,
    /// The accounts and sessions.
    pub accounts: Accounts,
    /// The rate limiters.
    pub rate_limits: Arc<RateLimits>,
    /// The catalogues of `/i18n`.
    pub catalogues: Arc<Catalogues>,
    /// The built app.
    pub static_app: Arc<StaticApp>,
    /// Where the app's own product events go (H13).
    pub events: Arc<dyn EventSink>,
    /// The support requests (H9).
    pub support: Support,
}

impl AppState {
    /// Reads the catalogues and the built app, and gathers the rest around `backends`.
    ///
    /// # Errors
    ///
    /// When the catalogues or the app cannot be read.
    pub fn new(config: Config, backends: Backends) -> Result<Self, StartError> {
        let catalogues = Catalogues::load(&config.i18n_dir, &config.legal)?;
        let static_app = StaticApp::load(&config.app_dir)?;
        let settings = accounts::settings(&config, catalogues.languages().to_vec());
        let library = library(&backends, &config);
        let support = support(&backends);
        Ok(Self {
            config: Arc::new(config),
            readiness: backends.readiness,
            library_store: backends.library_store,
            library,
            accounts: Accounts::new(backends.accounts, settings),
            rate_limits: Arc::new(RateLimits::new()),
            catalogues: Arc::new(catalogues),
            static_app: Arc::new(static_app),
            events: backends.events,
            support,
        })
    }
}

/// The library's use cases over the hosted store of `backends`, with the accounts' clock, ids and
/// product events, and the plans of `config`.
fn library(backends: &Backends, config: &Config) -> Library {
    let ports = LibraryPorts {
        store: Arc::clone(&backends.library_store),
        clock: Arc::clone(&backends.accounts.clock),
        ids: Arc::clone(&backends.accounts.ids),
        events: Arc::clone(&backends.accounts.events),
    };
    Library::new(ports, config.plans)
}

/// The support use cases over the stores of `backends`, with the accounts' clock, ids and
/// product events.
fn support(backends: &Backends) -> Support {
    Support::new(SupportPorts {
        stores: backends.support.clone(),
        clock: Arc::clone(&backends.accounts.clock),
        ids: Arc::clone(&backends.accounts.ids),
        events: Arc::clone(&backends.accounts.events),
    })
}
