//! `serve`: the migrations, then the three listeners, until `SIGTERM` or Ctrl-C.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use metrics_exporter_prometheus::{BuildError, PrometheusHandle};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

use crate::app;
use crate::config::Config;
use crate::database::{self, DatabaseError, DatabaseProbe};
use crate::http::rate_limit::RATE_LIMIT_UPKEEP_PERIOD;
use crate::state::{AppState, StartError};
use crate::telemetry::{self, METRICS_UPKEEP_PERIOD};

/// Why the server stopped.
#[derive(Debug, Error)]
pub enum ServeError {
    /// The migrations failed.
    #[error(transparent)]
    Database(#[from] DatabaseError),
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
/// When the database does not answer, the state cannot be built, or a listener fails.
pub async fn serve(config: Config) -> Result<(), ServeError> {
    database::migrate(&config.database).await?;
    let readiness = Arc::new(DatabaseProbe::new(&config.database));
    let metrics = telemetry::recorder()?;
    let addresses = [config.http_addr, config.metrics_addr, config.admin_api_addr];
    let state = AppState::new(config, readiness)?;
    spawn_upkeep(&state, metrics.clone());
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
