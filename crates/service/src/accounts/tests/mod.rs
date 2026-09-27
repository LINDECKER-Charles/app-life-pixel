//! The accounts use cases over the in-memory adapters.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

mod account_data;
mod emails;
mod passwords;
mod sessions;
mod sign_in;
mod sign_up;

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;
use time::macros::datetime;

use super::memory::{
    InMemoryAccountStore, InMemoryEmailTokenStore, InMemorySessionStore, RecordingMailer,
};
use super::ports::{Email, Message};
use super::{
    Accounts, AccountsPorts, AccountsSettings, Credentials, PasswordHashing, SignUp, SignedIn,
};
use crate::memory::{FixedClock, RecordingEvents, SequentialIds};
use crate::plans::Plans;

/// When the tests start.
const START: OffsetDateTime = datetime!(2026-09-01 12:00 UTC);
/// The origin of the links in emails.
const PUBLIC_URL: &str = "https://life-pixel.test";
/// The address signed up with.
const EMAIL: &str = "ada@example.com";
/// The password signed up with.
const PASSWORD: &str = "correct horse battery";
/// Another password, long enough.
const OTHER_PASSWORD: &str = "staple engine lantern";
/// The free plan's quota in these tests.
const FREE_STORAGE_BYTES: u64 = 1_000_000;
/// How long a test waits for an email sent on a task of its own.
const MAIL_TIMEOUT: Duration = Duration::from_secs(5);

/// The use cases and the adapters they work through.
struct Harness {
    accounts: Accounts,
    store: Arc<InMemoryAccountStore>,
    sessions: Arc<InMemorySessionStore>,
    tokens: Arc<InMemoryEmailTokenStore>,
    mailer: Arc<RecordingMailer>,
    clock: Arc<FixedClock>,
    events: Arc<RecordingEvents>,
}

impl Harness {
    /// The use cases with today's hashing parameters.
    fn new() -> Self {
        Self::hashing(PasswordHashing::standard())
    }

    /// The use cases hashing with `hashing`.
    fn hashing(hashing: PasswordHashing) -> Self {
        let store = Arc::new(InMemoryAccountStore::new());
        let sessions = Arc::new(InMemorySessionStore::new());
        let tokens = Arc::new(InMemoryEmailTokenStore::new());
        let mailer = Arc::new(RecordingMailer::new());
        let clock = Arc::new(FixedClock::new(START));
        let events = Arc::new(RecordingEvents::new());
        let ports = AccountsPorts {
            accounts: store.clone(),
            sessions: sessions.clone(),
            email_tokens: tokens.clone(),
            mailer: mailer.clone(),
            clock: clock.clone(),
            ids: Arc::new(SequentialIds::new()),
            events: events.clone(),
        };
        let accounts = Accounts::new(ports, settings(hashing));
        Self {
            accounts,
            store,
            sessions,
            tokens,
            mailer,
            clock,
            events,
        }
    }

    /// Signs up with [`EMAIL`] and [`PASSWORD`], in English.
    async fn sign_up(&self) -> SignedIn {
        self.accounts
            .sign_up(sign_up(EMAIL, PASSWORD))
            .await
            .unwrap()
    }

    /// Signs in with `email` and `password`.
    async fn sign_in(&self, email: &str, password: &str) -> Result<SignedIn, super::AccountsError> {
        let credentials = Credentials {
            email: email.to_owned(),
            password: password.to_owned(),
        };
        self.accounts.sign_in(credentials).await
    }

    /// The emails sent, once there are `count`.
    async fn emails(&self, count: usize) -> Vec<Email> {
        let sent = self.mailer.wait_for(count, MAIL_TIMEOUT).await;
        assert_eq!(sent.len(), count, "{sent:?}");
        sent
    }

    /// The names of the product events recorded so far.
    fn event_names(&self) -> Vec<&'static str> {
        self.events
            .events()
            .iter()
            .map(|event| event.name)
            .collect()
    }
}

/// The settings of the tests.
fn settings(hashing: PasswordHashing) -> AccountsSettings {
    AccountsSettings {
        public_url: PUBLIC_URL.to_owned(),
        languages: vec!["en".to_owned(), "fr".to_owned()],
        plans: Plans {
            free_storage_bytes: FREE_STORAGE_BYTES,
            free_mcp_calls_per_day: 0,
        },
        hashing,
    }
}

/// A sign-up in English.
fn sign_up(email: &str, password: &str) -> SignUp {
    SignUp {
        email: email.to_owned(),
        password: password.to_owned(),
        language: "en".to_owned(),
    }
}

/// The token of the link in `message`, after `path`.
fn token_in(message: &Message, path: &str) -> String {
    let link = message.link().unwrap();
    let token = link.strip_prefix(&format!("{PUBLIC_URL}{path}")).unwrap();
    token.to_owned()
}
