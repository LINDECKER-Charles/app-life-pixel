//! The auth routes over the local stack (H5): the Postgres stores of a test database, and the
//! SMTP mailer sending to Mailpit, where each test reads the emails of a recipient of its own.

mod emails;
mod routes;
mod sessions;
mod timing;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Method, Request};
use life_pixel_server::accounts;
use life_pixel_server::app;
use life_pixel_server::mail::{EmailTemplates, SmtpMailer};
use life_pixel_server::state::{AppState, Backends};
use life_pixel_server::testing::{MailMessage, TestDatabase, TestMailbox, load_test_env};
use life_pixel_service::memory::{InMemoryLibraryStore, RecordingEvents};
use life_pixel_service::support::memory::in_memory_stores;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::router::auth::{APP_ORIGIN, Browser, PASSWORD, from_app, sign_in, sign_up, with_json};
use crate::router::{Answer, Database, built_app, client_peer, local_env, read_config, send};

/// How long a test waits for an email to reach Mailpit.
const MAIL_WAIT: Duration = Duration::from_secs(15);
/// How often it asks Mailpit again.
const MAIL_POLL: Duration = Duration::from_millis(100);
/// The variable naming the local stack's SMTP server.
const SMTP_URL: &str = "LP_SMTP_URL";

/// The server's state over a test database and Mailpit.
pub struct AccountsStack {
    pub database: TestDatabase,
    pub state: AppState,
    _app_dir: TempDir,
}

/// The hosted accounts' ports over `database`, sending with `mailer`, and the other backends in
/// memory.
fn backends(database: &TestDatabase, mailer: SmtpMailer) -> Backends {
    let events: Arc<dyn life_pixel_service::ports::EventSink> = Arc::new(RecordingEvents::new());
    Backends {
        readiness: Arc::new(Database { answers: true }),
        library_store: Arc::new(InMemoryLibraryStore::new()),
        accounts: accounts::hosted_ports(database.pool(), Arc::new(mailer), Arc::clone(&events)),
        events,
        support: in_memory_stores(),
    }
}

impl AccountsStack {
    /// A new test database, the local configuration with the stack's SMTP server, and the
    /// hosted accounts' ports.
    pub async fn new() -> Self {
        load_test_env();
        let database = TestDatabase::create().await.unwrap();
        let app_dir = built_app();
        let mut env = local_env(app_dir.path());
        if let Ok(url) = std::env::var(SMTP_URL) {
            env.insert(SMTP_URL.to_owned(), url);
        }
        let config = read_config(&env).unwrap();
        let templates = EmailTemplates::load(&config.i18n_dir).unwrap();
        let mailer = SmtpMailer::new(&config.mail, templates).unwrap();
        let backends = backends(&database, mailer);
        let state = AppState::new(config, backends).unwrap();
        Self {
            database,
            state,
            _app_dir: app_dir,
        }
    }

    /// The answer to `request`, from the usual peer.
    pub async fn send(&self, request: Request<Body>) -> Answer {
        self.send_from(client_peer(), request).await
    }

    /// The answer to `request`, from `peer`.
    pub async fn send_from(&self, peer: SocketAddr, request: Request<Body>) -> Answer {
        let router = app::public_router(self.state.clone()).layer(MockConnectInfo(peer));
        send(router, request).await
    }

    /// The status of the answer to `request`.
    pub async fn status(&self, request: Request<Body>) -> u16 {
        self.send(request).await.status.as_u16()
    }

    /// Asks for a reset link for `mailbox`'s address, then reads its token in the email, in
    /// `language`: once per recipient, as the most recent reset email is read.
    pub async fn reset_token(&self, mailbox: &TestMailbox, language: &str) -> String {
        let request = post("password-reset", &json!({ "email": mailbox.address() }));
        assert_eq!(self.status(request).await, 202);
        let subject = text(language, "email.reset_password.subject");
        token_in(&message(mailbox, &subject).await, "/reset-password/confirm")
    }

    /// Signs `mailbox`'s address up in `language`: its session.
    pub async fn sign_up(&self, mailbox: &TestMailbox, language: &str) -> Browser {
        let answer = self.send(sign_up(mailbox.address(), language)).await;
        assert_eq!(answer.status, 201, "{:?}", answer.body);
        Browser::of(&answer)
    }

    /// Signs `mailbox`'s address in with [`PASSWORD`]: another session.
    pub async fn sign_in(&self, mailbox: &TestMailbox) -> Browser {
        let answer = self.send(sign_in(mailbox.address(), PASSWORD)).await;
        assert_eq!(answer.status, 200, "{:?}", answer.body);
        Browser::of(&answer)
    }

    /// Runs `statement` on the test database, with `address` as its `$1`.
    pub async fn execute(&self, statement: &'static str, address: &str) -> u64 {
        let query = sqlx::query(statement).bind(address);
        let done = query.execute(self.database.pool()).await.unwrap();
        done.rows_affected()
    }

    /// The value `query` selects, with `address` as its `$1`.
    pub async fn select<T>(&self, query: &'static str, address: &str) -> T
    where
        T: for<'row> sqlx::Decode<'row, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Unpin,
    {
        let query = sqlx::query_scalar(query).bind(address);
        query.fetch_one(self.database.pool()).await.unwrap()
    }
}

/// A `POST` of `body` to `/api/v1/auth/<route>` from the app, without a session.
pub fn post(route: &str, body: &Value) -> Request<Body> {
    with_json(
        from_app(Method::POST, &format!("/api/v1/auth/{route}")),
        body,
    )
}

/// The `count` most recent messages to `mailbox` titled `subject`, waiting for them.
pub async fn messages(mailbox: &TestMailbox, subject: &str, count: usize) -> Vec<MailMessage> {
    let deadline = Instant::now() + MAIL_WAIT;
    loop {
        let all = mailbox.messages().await.unwrap();
        let titled: Vec<_> = all.into_iter().filter(|m| m.subject == subject).collect();
        if titled.len() >= count {
            return titled;
        }
        assert!(
            Instant::now() < deadline,
            "{count} × {subject:?} to {}",
            mailbox.address()
        );
        tokio::time::sleep(MAIL_POLL).await;
    }
}

/// The most recent message to `mailbox` titled `subject`, waiting for it.
pub async fn message(mailbox: &TestMailbox, subject: &str) -> MailMessage {
    messages(mailbox, subject, 1).await.remove(0)
}

/// The text of `key` in the repository's catalogue of `language`.
pub fn text(language: &str, key: &str) -> String {
    let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
    let catalogue: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    catalogue[key].as_str().unwrap().to_owned()
}

/// The token of the link to `path` that `message` carries, checked to be the same in its text
/// and in its HTML, as an anchor.
pub fn token_in(message: &MailMessage, path: &str) -> String {
    let start = format!("{APP_ORIGIN}{path}?token=");
    let at = message
        .text
        .find(&start)
        .unwrap_or_else(|| panic!("{start} in {}", message.text));
    let rest = &message.text[at + start.len()..];
    let token: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    assert_eq!(token.len(), 43, "{}", message.text);
    let anchor = format!("<a href=\"{start}{token}\">");
    assert!(
        message.html.contains(&anchor),
        "{anchor} in {}",
        message.html
    );
    token
}
