//! The accounts' ports of the router tests: in-memory stores, a recording mailer and a clock
//! that stands still, each kept at hand to inspect or adjust.

use std::sync::Arc;
use std::time::Duration;

use life_pixel_service::AccountId;
use life_pixel_service::accounts::AccountsPorts;
use life_pixel_service::accounts::memory::{
    InMemoryAccountStore, InMemoryEmailTokenStore, InMemorySessionStore, RecordingMailer,
};
use life_pixel_service::accounts::ports::Email;
use life_pixel_service::memory::{FixedClock, RecordingEvents, SequentialIds};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

/// How long a test waits for an email.
pub const MAIL_TIMEOUT: Duration = Duration::from_secs(5);

/// The in-memory adapters behind a test server's accounts.
#[derive(Clone)]
pub struct TestAccounts {
    pub store: Arc<InMemoryAccountStore>,
    pub sessions: Arc<InMemorySessionStore>,
    pub tokens: Arc<InMemoryEmailTokenStore>,
    pub mailer: Arc<RecordingMailer>,
    pub clock: Arc<FixedClock>,
}

impl TestAccounts {
    /// Empty stores, and a clock showing the time of the call.
    pub fn new() -> Self {
        Self {
            store: Arc::new(InMemoryAccountStore::new()),
            sessions: Arc::new(InMemorySessionStore::new()),
            tokens: Arc::new(InMemoryEmailTokenStore::new()),
            mailer: Arc::new(RecordingMailer::new()),
            clock: Arc::new(FixedClock::new(OffsetDateTime::now_utc())),
        }
    }

    /// The ports over these adapters.
    pub fn ports(&self) -> AccountsPorts {
        AccountsPorts {
            accounts: self.store.clone(),
            sessions: self.sessions.clone(),
            email_tokens: self.tokens.clone(),
            mailer: self.mailer.clone(),
            clock: self.clock.clone(),
            ids: Arc::new(SequentialIds::new()),
            events: Arc::new(RecordingEvents::new()),
        }
    }
}

impl TestAccounts {
    /// The `count`th email sent, waiting for it.
    pub async fn email(&self, count: usize) -> Email {
        let sent = self.mailer.wait_for(count, MAIL_TIMEOUT).await;
        assert!(
            sent.len() >= count,
            "{} emails sent, not {count}",
            sent.len()
        );
        sent[count - 1].clone()
    }

    /// The token of the link of the `count`th email sent.
    pub async fn token(&self, count: usize) -> String {
        token_of(self.email(count).await.message.link().unwrap())
    }

    /// Suspends the account whose id an answer wrote as `id`.
    pub fn suspend(&self, id: &Value) {
        self.store.suspend(account_id(id));
    }
}

/// The token of an emailed `link`.
pub fn token_of(link: &str) -> String {
    link.split_once("?token=").unwrap().1.to_owned()
}

/// The account id an answer wrote as `id`.
pub fn account_id(id: &Value) -> AccountId {
    AccountId::from_uuid(Uuid::parse_str(id.as_str().unwrap()).unwrap())
}
