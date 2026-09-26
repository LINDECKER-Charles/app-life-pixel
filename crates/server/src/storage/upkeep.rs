//! The storage's periodic tasks, for `serve`: its metrics every 30 seconds, the sweeper every six
//! hours.

use std::future::Future;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time::MissedTickBehavior;

use super::metrics::{self, STORAGE_METRICS_PERIOD};
use super::sweeper::{SWEEP_PERIOD, Sweeper};

/// Starts the storage metrics and the sweeper, on the current runtime.
pub fn spawn_upkeep(pool: PgPool, sweeper: Sweeper) {
    every(STORAGE_METRICS_PERIOD, move || {
        let pool = pool.clone();
        async move {
            if let Err(error) = metrics::record(&pool).await {
                tracing::warn!(%error, "the storage metrics cannot be read");
            }
        }
    });
    every(SWEEP_PERIOD, move || {
        let sweeper = sweeper.clone();
        async move {
            match sweeper.sweep().await {
                Ok(deleted) => tracing::info!(deleted, "the orphaned objects are swept"),
                Err(error) => tracing::warn!(%error, "the sweep stopped"),
            }
        }
    });
}

/// Runs `task` now, then every `period`, one run at a time.
fn every<F>(period: Duration, task: impl Fn() -> F + Send + 'static)
where
    F: Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            task().await;
        }
    });
}
