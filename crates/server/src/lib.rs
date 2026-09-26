//! The Life Pixel server: the HTTP API under `/api/v1`, the i18n endpoint and the built app on
//! the public listener; `/metrics` and the internal admin API on two private ones. It translates
//! HTTP into `service` calls and nothing more. One line per module; with `stack-tests`, the
//! `testing` module of the tests that need the local stack.

pub mod accounts;
pub mod app;
pub mod commands;
pub mod config;
pub mod database;
pub mod events;
pub mod http;
pub mod mail;
pub mod openapi;
pub mod readiness;
pub mod routes;
pub mod state;
pub mod storage;
pub mod support;
pub mod telemetry;

#[cfg(feature = "stack-tests")]
pub mod testing;
