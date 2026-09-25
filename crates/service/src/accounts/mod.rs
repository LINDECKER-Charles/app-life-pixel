//! Accounts and sessions (H5): signing up, verifying the address, signing in and out, resetting
//! and changing the password, as docs/security-model.md and D29 describe them.
//!
//! [`Accounts`] holds the use cases, over four ports of their own — [`AccountStore`],
//! [`SessionStore`], [`EmailTokenStore`] and [`Mailer`] — and the clock, ids and product events of
//! [`crate::ports`]. Passwords are hashed with Argon2id on a blocking thread
//! ([`PasswordHashing`]); sessions and emailed links carry 32 random bytes, stored as their
//! SHA-256 ([`SecretToken`]). Emails leave on a task of their own: a mail server never slows a
//! request down, and a password reset takes the same time whether the address has an account or
//! not. With the `testing` feature, [`memory`] holds in-memory adapters of the ports.
//!
//! [`AccountStore`]: ports::AccountStore
//! [`SessionStore`]: ports::SessionStore
//! [`EmailTokenStore`]: ports::EmailTokenStore
//! [`Mailer`]: ports::Mailer

mod account;
mod cases;
mod error;
mod hashing;
pub mod ports;
mod values;

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use time::{Duration, OffsetDateTime};

pub use account::{Account, Authenticated, SignedIn};
pub use cases::{Credentials, PasswordChange, PasswordReset, SignUp};
pub use error::AccountsError;
pub use hashing::{
    ARGON2_LANES, ARGON2_MEMORY_KIB, ARGON2_PASSES, PasswordHash, PasswordHashing, Verification,
};
pub use values::{
    EmailAddress, Language, Password, SECRET_TOKEN_BYTES, SecretToken, TOKEN_HASH_BYTES, TokenHash,
};

use self::ports::{
    AccountRecord, AccountStore, Email, EmailTokenStore, Mailer, Message, SessionStore,
};
use crate::library::Usage;
use crate::plans::Plans;
use crate::ports::{Clock, EventSink, IdGenerator, ProductEvent};

/// How long a session lives without being seen.
pub const SESSION_LIFETIME: Duration = Duration::days(30);
/// How long after it was last seen a session's expiry moves again.
pub const SESSION_EXTENSION_AFTER: Duration = Duration::days(1);
/// How long a link verifying an address works.
pub const VERIFY_EMAIL_LIFETIME: Duration = Duration::days(7);
/// How long a link resetting a password works.
pub const RESET_PASSWORD_LIFETIME: Duration = Duration::hours(1);

/// The path, after `LP_PUBLIC_URL`, of the page that verifies an address.
const VERIFY_EMAIL_PATH: &str = "/verify-email?token=";
/// The path, after `LP_PUBLIC_URL`, of the page that sets a new password.
const RESET_PASSWORD_PATH: &str = "/reset-password/confirm?token=";

/// The adapters the accounts use cases work through.
#[derive(Clone)]
pub struct AccountsPorts {
    /// Where accounts are kept.
    pub accounts: Arc<dyn AccountStore>,
    /// Where sessions are kept.
    pub sessions: Arc<dyn SessionStore>,
    /// Where the tokens of emailed links are kept.
    pub email_tokens: Arc<dyn EmailTokenStore>,
    /// How emails leave.
    pub mailer: Arc<dyn Mailer>,
    /// The time of sessions and tokens.
    pub clock: Arc<dyn Clock>,
    /// New account ids.
    pub ids: Arc<dyn IdGenerator>,
    /// Where product events go.
    pub events: Arc<dyn EventSink>,
}

/// What configuration decides for the accounts.
#[derive(Clone, Debug)]
pub struct AccountsSettings {
    /// The origin links in emails start with: `LP_PUBLIC_URL`, without a trailing slash.
    pub public_url: String,
    /// The language codes an account may choose: those of the catalogues.
    pub languages: Vec<String>,
    /// The plan values: the quota shown with an account.
    pub plans: Plans,
    /// How passwords are hashed.
    pub hashing: PasswordHashing,
}

/// The accounts use cases.
#[derive(Clone)]
pub struct Accounts {
    ports: AccountsPorts,
    settings: Arc<AccountsSettings>,
}

impl Accounts {
    /// The use cases over `ports`, with the values of configuration.
    #[must_use]
    pub fn new(ports: AccountsPorts, settings: AccountsSettings) -> Self {
        Self {
            ports,
            settings: Arc::new(settings),
        }
    }

    fn now(&self) -> OffsetDateTime {
        self.ports.clock.now()
    }

    fn hashing(&self) -> &PasswordHashing {
        &self.settings.hashing
    }

    /// The account of `record`, as its owner sees it.
    fn view(&self, record: &AccountRecord) -> Account {
        Account {
            id: record.id,
            email: record.email.clone(),
            is_email_verified: record.email_verified_at.is_some(),
            language: record.language.clone(),
            plan: record.plan.clone(),
            storage: Usage {
                used_bytes: record.storage_used_bytes,
                limit_bytes: Some(self.settings.plans.free_storage_bytes),
            },
            created_at: record.created_at,
        }
    }

    /// The link to the page at `path`, carrying `token`.
    fn link(&self, path: &str, token: &SecretToken) -> String {
        format!("{}{path}{}", self.settings.public_url, token.expose())
    }

    /// Records the event `name` about `account`.
    fn record(&self, name: &'static str, account: &AccountRecord) {
        self.ports.events.record(ProductEvent {
            name,
            account: Some(account.id),
            properties: Vec::new(),
        });
    }

    /// Sends `message` to `account` on a task of its own; a failure is logged, never returned.
    fn dispatch(&self, account: &AccountRecord, message: Message) {
        let mailer = Arc::clone(&self.ports.mailer);
        let key = message.key();
        let email = Email {
            to: account.email.clone(),
            language: account.language.clone(),
            message,
        };
        tokio::spawn(async move {
            if let Err(error) = mailer.send(email).await {
                tracing::warn!(%error, message = key, "an account email was not sent");
            }
        });
    }
}
