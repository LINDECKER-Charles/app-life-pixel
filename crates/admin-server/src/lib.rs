//! The Life Pixel admin server: the admin console's API under `/api/admin/v1` — admin accounts
//! with a password and TOTP, the relay to the server's internal admin API, monitoring —, its
//! catalogues and the built console on the console listener; `/metrics` on a private one. It
//! depends on no domain crate. One line per module; with `stack-tests`, the `testing` module of
//! the tests that need the local stack.

pub mod admins;
pub mod app;
pub mod commands;
pub mod config;
pub mod database;
pub mod http;
pub mod monitoring;
pub mod openapi;
pub mod relay;
pub mod state;
pub mod telemetry;

#[cfg(feature = "stack-tests")]
pub mod testing;
