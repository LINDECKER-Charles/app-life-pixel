//! The accounts' stores on Postgres: accounts, sessions and the tokens of emailed links, as the
//! ports of `service::accounts` describe them. Tokens are keyed by their SHA-256.

mod account_store;
mod email_token_store;
mod rows;
mod session_store;

pub use account_store::PostgresAccountStore;
pub use email_token_store::PostgresEmailTokenStore;
pub use session_store::PostgresSessionStore;
