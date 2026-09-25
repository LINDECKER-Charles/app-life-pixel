//! The rate-limit policies of docs/v1/server.md, and what their limits count requests by.

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use life_pixel_service::AccountId;

/// A rate-limit policy. The routes of each apply it; `Api` covers every other API route.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Policy {
    /// Signing in (H5): 10 a minute per address, 5 a minute per email.
    SignIn,
    /// Signing up (H5): 5 an hour per address.
    SignUp,
    /// Asking for a password reset (H5): 5 an hour per address and per email.
    PasswordReset,
    /// Resending the verification email (H5): 3 an hour per account.
    VerificationResend,
    /// Product events (H13): 60 a minute per address.
    Events,
    /// Support requests (H9): 10 a day per account.
    SupportCreate,
    /// MCP calls (A3): 120 a minute per token.
    Mcp,
    /// Every other API route: 600 a minute per account, else per address.
    Api,
}

/// What a limit counts requests by.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RateKey {
    /// The client's address.
    Address(IpAddr),
    /// An email address, lowercased: see [`RateKey::email`].
    Email(String),
    /// A signed-in account.
    Account(AccountId),
    /// An access token, by an identifier that is not the secret itself.
    Token(String),
}

impl RateKey {
    /// The key of an email address, whatever its case and surrounding spaces.
    #[must_use]
    pub fn email(address: &str) -> Self {
        Self::Email(address.trim().to_lowercase())
    }

    pub(super) fn kind(&self) -> KeyKind {
        match self {
            Self::Address(_) => KeyKind::Address,
            Self::Email(_) => KeyKind::Email,
            Self::Account(_) => KeyKind::Account,
            Self::Token(_) => KeyKind::Token,
        }
    }
}

/// The kind of a [`RateKey`], without its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum KeyKind {
    Address,
    Email,
    Account,
    Token,
}

const MINUTE: Duration = Duration::from_secs(60);
const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// `count` requests per `window`, for one kind of key of one policy.
pub(super) struct Limit {
    pub policy: Policy,
    pub key: KeyKind,
    pub count: NonZeroU32,
    pub window: Duration,
}

const fn limit(policy: Policy, key: KeyKind, per_window: (u32, Duration)) -> Limit {
    let Some(count) = NonZeroU32::new(per_window.0) else {
        panic!("a limit allows at least one request");
    };
    Limit {
        policy,
        key,
        count,
        window: per_window.1,
    }
}

/// The table of docs/v1/server.md: one line per policy and key.
pub(super) const LIMITS: &[Limit] = &[
    limit(Policy::SignIn, KeyKind::Address, (10, MINUTE)),
    limit(Policy::SignIn, KeyKind::Email, (5, MINUTE)),
    limit(Policy::SignUp, KeyKind::Address, (5, HOUR)),
    limit(Policy::PasswordReset, KeyKind::Address, (5, HOUR)),
    limit(Policy::PasswordReset, KeyKind::Email, (5, HOUR)),
    limit(Policy::VerificationResend, KeyKind::Account, (3, HOUR)),
    limit(Policy::Events, KeyKind::Address, (60, MINUTE)),
    limit(Policy::SupportCreate, KeyKind::Account, (10, DAY)),
    limit(Policy::Mcp, KeyKind::Token, (120, MINUTE)),
    limit(Policy::Api, KeyKind::Account, (600, MINUTE)),
    limit(Policy::Api, KeyKind::Address, (600, MINUTE)),
];
