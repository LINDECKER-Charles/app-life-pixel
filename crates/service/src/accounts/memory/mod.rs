//! In-memory adapters of the accounts ports, for tests: feature `testing`.

mod account_store;
mod email_token_store;
mod mailer;
mod session_store;

pub use account_store::InMemoryAccountStore;
pub use email_token_store::InMemoryEmailTokenStore;
pub use mailer::RecordingMailer;
pub use session_store::InMemorySessionStore;
