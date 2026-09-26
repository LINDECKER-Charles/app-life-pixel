//! `/api/v1/tokens` over the local stack (A3): creation with the secret shown once and the MCP
//! server to register, the checks and the limit, the listing, and revocation. Behind the
//! `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "tokens_api/creation.rs"]
mod creation;
#[path = "tokens_api/listing.rs"]
mod listing;
#[path = "common/mod.rs"]
mod router;
#[path = "common/stack.rs"]
mod stack;
#[path = "common/tokens.rs"]
mod tokens;

/// The path of the tokens, under `/api/v1`.
pub const TOKENS: &str = "/tokens";
