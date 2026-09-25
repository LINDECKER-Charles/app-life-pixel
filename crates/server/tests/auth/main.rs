//! The auth routes over in-memory accounts (H5): the session cookie, sliding expiry, CSRF, the
//! problems of each route and `auth_events_total`. The stack tests run them again over Postgres
//! and Mailpit.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "../common/mod.rs"]
mod common;
mod csrf;
mod metrics;
mod passwords;
mod sessions;
mod sign_in;
mod sign_up;
mod verification;
