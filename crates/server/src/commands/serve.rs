//! `serve`: the migrations, then the three listeners, until `SIGTERM` or Ctrl-C.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use life_pixel_service::accounts::ports::Mailer;
use metrics_exporter_prometheus::{BuildError, PrometheusHandle};
use object_store::ObjectStore;
use sqlx::PgPool;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

use crate::accounts;
use crate::app;
use crate::config::Config;
use crate::database::{self, DatabaseError, DatabaseReadiness};
use crate::events::{self, PostgresEventSink};
use crate::http::rate_limit::RATE_LIMIT_UPKEEP_PERIOD;
use crate::mail::{EmailTemplates, MailSetupError, SmtpMailer, TemplateError};
use crate::routes;
use crate::state::{AppState, Backends, StartError};
use crate::storage::{self, HostedLibraryStore, ObjectStoreSetupError, Sweeper};
use crate::telemetry::{self, METRICS_UPKEEP_PERIOD};

/// Why the server stopped.
#[derive(Debug, Error)]
pub enum ServeError {
    /// The database does not answer, or the migrations failed.
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// The object storage cannot be set up.
    #[error(transparent)]
    Objects(#[from] ObjectStoreSetupError),
    /// The mail server's URL or the sender is invalid.
    #[error(transparent)]
    Mail(#[from] MailSetupError),
    /// The emails' texts could not be read.
    #[error("LP_I18N_DIR: {0}")]
    Templates(#[from] TemplateError),
    /// The metrics recorder could not be installed.
    #[error("the metrics recorder cannot be installed: {0}")]
    Metrics(#[from] BuildError),
    /// The catalogues or the app could not be read.
    #[error(transparent)]
    Start(#[from] StartError),
    /// A listener's address could not be bound.
    #[error("cannot listen on {address}: {source}")]
    Bind {
        /// The address.
        address: SocketAddr,
        /// The failure.
        source: io::Error,
    },
    /// A listener failed.
    #[error("the listener on {address} failed: {source}")]
    Listen {
        /// The address.
        address: SocketAddr,
        /// The failure.
        source: io::Error,
    },
}

/// Runs the migrations, then serves the public, metrics and internal admin listeners until the
/// process is asked to stop, letting the requests under way finish.
///
/// # Errors
///
/// When the database does not answer, the object storage or the mailer cannot be set up, the
/// state cannot be built, or a listener fails.
pub async fn serve(config: Config) -> Result<(), ServeError> {
    let (pool, objects) = open_storage(&config).await?;
    let metrics = telemetry::recorder()?;
    storage::metrics::describe();
    accounts::metrics::describe();
    routes::library::metrics::describe();
    events::metrics::describe();
    let events_sink = PostgresEventSink::spawn(pool.clone(), config.secrets.events.clone());
    let backends = Backends {
        readiness: Arc::new(DatabaseReadiness::new(pool.clone())),
        library_store: Arc::new(HostedLibraryStore::new(pool.clone(), Arc::clone(&objects))),
        accounts: accounts::hosted_ports(&pool, mailer(&config)?, Arc::clone(&events_sink)),
        events: events_sink,
    };
    let addresses = [config.http_addr, config.metrics_addr, config.admin_api_addr];
    let state = AppState::new(config, backends)?;
    spawn_upkeep(&state, metrics.clone());
    accounts::spawn_purge(state.accounts.clone());
    events::spawn_purge(pool.clone());
    storage::spawn_upkeep(pool.clone(), Sweeper::new(pool, objects));
    let stop = stop_signal();
    let [public, private, admin] = addresses;
    tokio::try_join!(
        listen(public, app::public_router(state.clone()), stop.clone()),
        listen(private, app::metrics_router(metrics), stop.clone()),
        listen(admin, app::admin_router(state), stop),
    )?;
    tracing::info!("stopped");
    Ok(())
}

/// The migrated database's pool, and the object storage.
async fn open_storage(config: &Config) -> Result<(PgPool, Arc<dyn ObjectStore>), ServeError> {
    let pool = database::connect(&config.database).await?;
    database::migrate(&pool).await?;
    let objects = storage::object_store(&config.storage)?;
    Ok((pool, objects))
}

/// The SMTP mailer of `LP_SMTP_URL`, with the emails of the catalogues.
fn mailer(config: &Config) -> Result<Arc<dyn Mailer>, ServeError> {
    let templates = EmailTemplates::load(&config.i18n_dir)?;
    Ok(Arc::new(SmtpMailer::new(&config.mail, templates)?))
}

async fn listen(
    address: SocketAddr,
    router: Router,
    mut stop: watch::Receiver<bool>,
) -> Result<(), ServeError> {
    let listener = TcpListener::bind(address)
        .await
        .map_err(|source| ServeError::Bind { address, source })?;
    tracing::info!(%address, "listening");
    let service = router.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service)
        .with_graceful_shutdown(async move {
            let _stopped = stop.wait_for(|stop| *stop).await;
        })
        .await
        .map_err(|source| ServeError::Listen { address, source })
}

/// Forgets the idle rate-limit keys, and drains the metrics' histograms, periodically.
fn spawn_upkeep(state: &AppState, metrics: PrometheusHandle) {
    let rate_limits = Arc::clone(&state.rate_limits);
    every(RATE_LIMIT_UPKEEP_PERIOD, move || {
        rate_limits.retain_recent()
    });
    every(METRICS_UPKEEP_PERIOD, move || metrics.run_upkeep());
}

fn every(period: Duration, task: impl Fn() + Send + 'static) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            task();
        }
    });
}

/// Becomes `true` on `SIGTERM` or Ctrl-C.
fn stop_signal() -> watch::Receiver<bool> {
    let (sender, receiver) = watch::channel(false);
    tokio::spawn(async move {
        wait_for_signal().await;
        tracing::info!("stopping");
        let _unobserved = sender.send(true);
    });
    receiver
}

#[cfg(unix)]
async fn wait_for_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let terminate = async {
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => drop(terminate.recv().await),
            Err(error) => {
                tracing::warn!(%error, "SIGTERM cannot be caught");
                std::future::pending::<()>().await;
            }
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        () = terminate => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_signal() {
    let _interrupted = tokio::signal::ctrl_c().await;
}
