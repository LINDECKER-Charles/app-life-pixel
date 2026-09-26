//! Admin accounts: a password (Argon2id), a mandatory TOTP second factor, and short sessions.
//!
//! [`Admins`] signs in, authenticates a session's token, signs out, creates and disables admins;
//! [`routes`] are `/auth/sign-in`, `/auth/session` and `/auth/sign-out`; [`rate_limit`] allows 5
//! sign-in attempts a minute per client address and per email.

pub mod clock;
pub mod password;
pub mod rate_limit;
pub mod routes;
pub mod secret_box;
pub mod sessions;
pub mod sign_in;
pub mod store;
pub mod totp;

use std::sync::Arc;
use std::time::Duration as StdDuration;

use time::OffsetDateTime;
use tokio::time::MissedTickBehavior;

use clock::Clock;
use secret_box::SecretBox;
use sessions::SessionToken;
use store::{AdminIdentity, AdminStore};

pub use sign_in::{CreateAdminError, Credentials, SignInError, SignedIn};

/// The longest address an admin may have.
pub const EMAIL_MAX_CHARS: usize = 254;
/// How often the ended sessions are deleted.
pub const SESSION_PURGE_PERIOD: StdDuration = StdDuration::from_secs(60 * 60);

/// The admin accounts and their sessions.
#[derive(Clone)]
pub struct Admins {
    store: AdminStore,
    secrets: SecretBox,
    clock: Arc<dyn Clock>,
}

impl Admins {
    /// The accounts of `store`, their TOTP secrets sealed in `secrets`, on `clock`'s time.
    #[must_use]
    pub fn new(store: AdminStore, secrets: SecretBox, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            secrets,
            clock,
        }
    }

    /// Now, on the accounts' clock.
    #[must_use]
    pub fn now(&self) -> OffsetDateTime {
        self.clock.now()
    }

    /// The admin of `token`'s session when it is live: its admin not disabled, neither idle 30
    /// minutes nor 8 hours old. A live session's `last_seen_at` moves, once a minute at most.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn authenticate(
        &self,
        token: &SessionToken,
    ) -> Result<Option<AdminIdentity>, sqlx::Error> {
        let hash = token.hash();
        let Some(session) = self.store.find_session(&hash).await? else {
            return Ok(None);
        };
        let now = self.clock.now();
        if !session.times.is_live(now) {
            return Ok(None);
        }
        if session.times.is_due_for_touch(now) {
            self.store.touch_session(&hash, now).await?;
        }
        Ok(Some(session.admin))
    }

    /// Ends the session of `token`.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn sign_out(&self, token: &SessionToken) -> Result<(), sqlx::Error> {
        self.store.delete_session(&token.hash()).await
    }

    /// Disables the admin of `email` and ends its sessions: `false` when no active admin has it.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn disable(&self, email: &str) -> Result<bool, sqlx::Error> {
        self.store.disable(email.trim(), self.clock.now()).await
    }

    /// Deletes the sessions that ended, every hour, from now on.
    pub fn spawn_purge(&self) {
        let admins = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(SESSION_PURGE_PERIOD);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                match admins.store.purge_sessions(admins.clock.now()).await {
                    Ok(purged) => tracing::info!(purged, "ended admin sessions deleted"),
                    Err(error) => tracing::warn!(%error, "ended admin sessions not deleted"),
                }
            }
        });
    }
}
