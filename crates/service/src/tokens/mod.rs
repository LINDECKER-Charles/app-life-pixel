//! Personal access tokens (A3): what an MCP client authenticates with on the hosted endpoint, as
//! docs/mcp.md describes them — scoped, shown once, stored hashed, revocable and expiring, their
//! last use visible to their owner.
//!
//! [`Tokens`] holds the use cases over one port of their own, [`TokenStore`], and the clock and
//! ids of [`crate::ports`]. A token is `lp_pat_` followed by 32 random bytes in base64url
//! ([`AccessTokenSecret`]); only its SHA-256 is stored. With the `testing` feature, [`memory`]
//! holds an in-memory adapter of the port.
//!
//! [`TokenStore`]: ports::TokenStore

mod cases;
mod error;
pub mod ports;
mod values;

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use time::Duration;

pub use cases::{AccessToken, CreatedToken, NewToken};
pub use error::TokensError;
pub use values::{ACCESS_TOKEN_PREFIX, AccessTokenSecret, TokenScope};

use self::ports::TokenStore;
use crate::ports::{Clock, IdGenerator};

/// How long after its last recorded use a token's use is recorded again: at most once a minute.
pub const LAST_USE_PRECISION: Duration = Duration::minutes(1);

/// The adapters the token use cases work through.
#[derive(Clone)]
pub struct TokensPorts {
    /// Where tokens are kept.
    pub store: Arc<dyn TokenStore>,
    /// The time of creations, uses, expiries and revocations.
    pub clock: Arc<dyn Clock>,
    /// New token ids.
    pub ids: Arc<dyn IdGenerator>,
}

/// The personal access token use cases.
#[derive(Clone)]
pub struct Tokens {
    ports: TokensPorts,
}

impl Tokens {
    /// The use cases over `ports`.
    #[must_use]
    pub fn new(ports: TokensPorts) -> Self {
        Self { ports }
    }

    fn store(&self) -> &dyn TokenStore {
        self.ports.store.as_ref()
    }
}
