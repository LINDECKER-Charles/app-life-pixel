//! What the use cases return: an account as its owner sees it, and the sessions they open or
//! check.

use time::OffsetDateTime;

use super::values::{SecretToken, TokenHash};
use crate::ids::AccountId;
use crate::library::Usage;

/// An account as its owner sees it: never its password hash or status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    /// Its id.
    pub id: AccountId,
    /// Its address.
    pub email: String,
    /// Whether the address is verified.
    pub is_email_verified: bool,
    /// Its language code.
    pub language: String,
    /// Its plan.
    pub plan: String,
    /// Its documents' bytes and its quota.
    pub storage: Usage,
    /// When it signed up.
    pub created_at: OffsetDateTime,
}

/// A session just opened, by signing up or in.
#[derive(Clone, Debug)]
pub struct SignedIn {
    /// The session's secret, for the cookie alone.
    pub token: SecretToken,
    /// The account signed in.
    pub account: Account,
}

/// The session a request carries, once checked: it is live and its account active.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authenticated {
    /// The session's key.
    pub token_hash: TokenHash,
    /// The account signed in.
    pub account: Account,
    /// Whether it was last seen more than a day ago: its expiry should move, with
    /// [`Accounts::extend_session`](super::Accounts::extend_session), and its cookie be sent
    /// again.
    pub is_due_for_extension: bool,
}
