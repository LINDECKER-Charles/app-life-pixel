//! The Life Pixel server: the HTTP API under `/api/v1`, the i18n endpoint and the built app on
//! the public listener; `/metrics` and the internal admin API on two private ones. It translates
//! HTTP into `service` calls and nothing more. One line per module.

pub mod app;
pub mod commands;
pub mod config;
pub mod database;
pub mod http;
pub mod openapi;
pub mod readiness;
pub mod routes;
pub mod state;
pub mod telemetry;
