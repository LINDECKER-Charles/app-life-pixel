//! The accounts' daily task, for `serve`: the purge of expired sessions and emailed tokens.

use std::time::Duration;

use life_pixel_service::accounts::Accounts;
use tokio::time::MissedTickBehavior;

/// How often expired sessions and tokens are purged.
pub const PURGE_PERIOD: Duration = Duration::from_secs(24 * 60 * 60);

/// Purges now, then every day, on the current runtime.
pub fn spawn_purge(accounts: Accounts) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(PURGE_PERIOD);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            match accounts.purge_expired().await {
                Ok((sessions, tokens)) => {
                    tracing::info!(
                        sessions,
                        tokens,
                        "the expired sessions and tokens are purged"
                    );
                }
                Err(error) => tracing::warn!(%error, "the expired sessions were not purged"),
            }
        }
    });
}
