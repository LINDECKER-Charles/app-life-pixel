//! The stack tests: the server's hosted library, its accounts and sessions, its migrations, its
//! sweeper and the helpers of `life_pixel_server::testing`, against the local stack — Postgres,
//! S3Mock and Mailpit. Each test works on a database, a bucket prefix and a mail recipient of its
//! own. Behind the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

mod accounts;
mod common;
mod contract;
mod failing_objects;
mod migrations;
mod search;
mod sweeper;
mod test_support;
mod writes;

/// The router tests' helpers: the local configuration and the requests of the auth routes.
#[path = "../common/mod.rs"]
mod router;
