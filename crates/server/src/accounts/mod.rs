//! Accounts and sessions over HTTP (H5): the building blocks of every route that acts for a
//! signed-in account.
//!
//! - [`resolve`], on all of `/api/v1`, checks the session cookie once per request and leaves a
//!   [`SessionState`] in the request's extensions, with the `AccountId` of a live session for the
//!   `api` rate limit. It extends a session seen more than a day ago and sends its cookie again,
//!   and clears the cookie of a session that ended.
//! - [`CurrentSession`] is the extractor of a live session: without one, the route answers
//!   `auth.unauthenticated`, or `auth.account_suspended`.
//! - [`csrf::protect`], on the routes a session acts through, refuses an unsafe request carrying
//!   the cookie from another origin or without the session's `X-CSRF-Token`;
//!   [`csrf::check_origin`], on sign-up and sign-in, checks the origin alone.
//!
//! [`cookie`] reads, sets and clears `__Host-lp_session`; [`metrics`] counts
//! `auth_events_total{event}`; [`spawn_purge`] purges expired sessions and tokens daily.

pub mod cookie;
pub mod csrf;
pub mod metrics;
mod session;
mod upkeep;

use std::sync::Arc;

use life_pixel_service::accounts::ports::Mailer;
use life_pixel_service::accounts::{AccountsPorts, AccountsSettings, PasswordHashing};
use life_pixel_service::ports::{EventSink, SystemClock, UuidV7Ids};
use sqlx::PgPool;

pub use session::{CurrentSession, SessionState, resolve};
pub use upkeep::{PURGE_PERIOD, spawn_purge};

use crate::config::Config;
use crate::storage::{PostgresAccountStore, PostgresEmailTokenStore, PostgresSessionStore};

/// The accounts' ports of the hosted service: the stores of the database of `pool`, `mailer`,
/// the system's clock, UUIDv7s and `events` (H13).
#[must_use]
pub fn hosted_ports(
    pool: &PgPool,
    mailer: Arc<dyn Mailer>,
    events: Arc<dyn EventSink>,
) -> AccountsPorts {
    AccountsPorts {
        accounts: Arc::new(PostgresAccountStore::new(pool.clone())),
        sessions: Arc::new(PostgresSessionStore::new(pool.clone())),
        email_tokens: Arc::new(PostgresEmailTokenStore::new(pool.clone())),
        mailer,
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        events,
    }
}

/// The accounts' settings of `config`, an account choosing among `languages`.
#[must_use]
pub fn settings(config: &Config, languages: Vec<String>) -> AccountsSettings {
    AccountsSettings {
        public_url: config.public_url.as_str().to_owned(),
        languages,
        plans: config.plans,
        hashing: PasswordHashing::standard(),
    }
}
