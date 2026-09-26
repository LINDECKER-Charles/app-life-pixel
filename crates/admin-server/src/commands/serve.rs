//! `serve`: the migrations, then the console and metrics listeners, until `SIGTERM` or Ctrl-C.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use metrics_exporter_prometheus::{BuildError, PrometheusHandle};
use sqlx::PgPool;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

use crate::admins::clock::SystemClock;
use crate::admins::rate_limit::RATE_LIMIT_UPKEEP_PERIOD;
use crate::app;
use crate::config::Config;
use crate::database::{self, DatabaseError};
use crate::state::{AppState, StartError};
use crate::telemetry::{self, METRICS_UPKEEP_PERIOD};

/// Why the admin server stopped.
#[derive(Debug, Error)]
pub enum ServeError {
    /// The database does not answer, or the migrations failed.
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// The metrics recorder could not be installed.
    #[error("the metrics recorder cannot be installed: {0}")]
    Metrics(#[from] BuildError),
    /// The catalogues, the console or a client could not be set up.
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

/// Runs the migrations, then serves the console and metrics listeners until the process is
/// asked to stop, letting the requests under way finish.
///
/// # Errors
///
/// When the database does not answer, the state cannot be built, or a listener fails.
pub async fn serve(config: Config) -> Result<(), ServeError> {
    let pool = migrated_pool(&config).await?;
    let metrics = telemetry::recorder()?;
    let (console, private) = (config.http_addr, config.metrics_addr);
    let state = AppState::new(config, pool, Arc::new(SystemClock))?;
    spawn_upkeep(&state, metrics.clone());
    state.admins.spawn_purge();
    let stop = stop_signal();
    tokio::try_join!(
        listen(console, app::router(state), stop.clone()),
        listen(private, app::metrics_router(metrics), stop),
    )?;
    tracing::info!("stopped");
    Ok(())
}

/// The pool of the admin server's database, once migrated.
async fn migrated_pool(config: &Config) -> Result<PgPool, DatabaseError> {
    let pool = database::connect(&config.database_url).await?;
    database::migrate(&pool).await?;
    Ok(pool)
}

/// Forgets the idle sign-in rate-limit keys, and drains the metrics' histograms, periodically.
fn spawn_upkeep(state: &AppState, metrics: PrometheusHandle) {
    let limits = Arc::clone(&state.sign_in_limits);
    every(RATE_LIMIT_UPKEEP_PERIOD, move || limits.retain_recent());
    every(METRICS_UPKEEP_PERIOD, move || metrics.run_upkeep());
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
