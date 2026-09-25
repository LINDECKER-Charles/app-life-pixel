//! A session's life: checked on each request, extended once a day, ended by signing out, purged
//! once expired.

use crate::accounts::ports::{AccountStatus, SessionExtension};
use crate::accounts::values::{SecretToken, TokenHash};
use crate::accounts::{
    Accounts, AccountsError, Authenticated, SESSION_EXTENSION_AFTER, SESSION_LIFETIME,
};

impl Accounts {
    /// The session of `token`, when it is live and its account active. Whether its expiry is due
    /// to move is said, not done: [`Accounts::extend_session`] does it.
    ///
    /// # Errors
    ///
    /// `auth.unauthenticated` for an unknown or expired session, `auth.account_suspended`,
    /// `service.unavailable`.
    pub async fn authenticate(&self, token: &SecretToken) -> Result<Authenticated, AccountsError> {
        let token_hash = token.hash();
        let now = self.now();
        let session = self.ports.sessions.find(&token_hash).await?;
        let session = session
            .filter(|session| session.expires_at > now)
            .ok_or(AccountsError::Unauthenticated)?;
        let account = self.ports.accounts.get(session.account_id).await?;
        let account = account.ok_or(AccountsError::Unauthenticated)?;
        if account.status == AccountStatus::Suspended {
            return Err(AccountsError::AccountSuspended);
        }
        Ok(Authenticated {
            token_hash,
            account: self.view(&account),
            is_due_for_extension: now - session.last_seen_at > SESSION_EXTENSION_AFTER,
        })
    }

    /// Moves the expiry of the session of `token_hash` `SESSION_LIFETIME` after now.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn extend_session(&self, token_hash: &TokenHash) -> Result<(), AccountsError> {
        let now = self.now();
        let extension = SessionExtension {
            seen_at: now,
            expires_at: now + SESSION_LIFETIME,
        };
        Ok(self.ports.sessions.extend(token_hash, extension).await?)
    }

    /// Ends the session of `token`, if it exists.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn sign_out(&self, token: &SecretToken) -> Result<(), AccountsError> {
        Ok(self.ports.sessions.delete(&token.hash()).await?)
    }

    /// Deletes the expired sessions and emailed tokens: the daily purge. Returns how many of each
    /// were deleted.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn purge_expired(&self) -> Result<(u64, u64), AccountsError> {
        let now = self.now();
        let sessions = self.ports.sessions.purge_expired(now).await?;
        let tokens = self.ports.email_tokens.purge_expired(now).await?;
        Ok((sessions, tokens))
    }
}
