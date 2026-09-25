//! The accounts use cases, one module per flow, and what several of them share: opening a
//! session, and issuing an emailed token.

mod password_change;
mod password_reset;
mod profile;
mod sessions;
mod sign_in;
mod sign_up;
mod verification;

pub use password_change::PasswordChange;
pub use password_reset::PasswordReset;
pub use sign_in::Credentials;
pub use sign_up::SignUp;

use time::Duration;

use super::ports::{AccountRecord, EmailTokenRecord, SessionRecord, TokenPurpose, TokenUse};
use super::values::SecretToken;
use super::{
    Accounts, AccountsError, RESET_PASSWORD_LIFETIME, SESSION_LIFETIME, SignedIn,
    VERIFY_EMAIL_LIFETIME,
};

/// How long a token for `purpose` works.
fn lifetime(purpose: TokenPurpose) -> Duration {
    match purpose {
        TokenPurpose::VerifyEmail => VERIFY_EMAIL_LIFETIME,
        TokenPurpose::ResetPassword => RESET_PASSWORD_LIFETIME,
    }
}

impl Accounts {
    /// Opens a session for `account`.
    async fn open_session(&self, account: &AccountRecord) -> Result<SignedIn, AccountsError> {
        let token = SecretToken::generate();
        let now = self.now();
        let session = SessionRecord {
            token_hash: token.hash(),
            account_id: account.id,
            created_at: now,
            last_seen_at: now,
            expires_at: now + SESSION_LIFETIME,
        };
        self.ports.sessions.create(session).await?;
        Ok(SignedIn {
            token,
            account: self.view(account),
        })
    }

    /// Stores a new token for `purpose`, working for as long as its purpose allows, and returns
    /// it.
    async fn issue_token(
        &self,
        account: &AccountRecord,
        purpose: TokenPurpose,
    ) -> Result<SecretToken, AccountsError> {
        let token = SecretToken::generate();
        let now = self.now();
        let record = EmailTokenRecord {
            token_hash: token.hash(),
            account_id: account.id,
            purpose,
            created_at: now,
            expires_at: now + lifetime(purpose),
        };
        self.ports.email_tokens.create(record).await?;
        Ok(token)
    }

    /// The account `token` acts for, once it is spent for `purpose`.
    async fn spend_token(
        &self,
        token: &str,
        purpose: TokenPurpose,
    ) -> Result<AccountRecord, AccountsError> {
        let token = SecretToken::parse(token).ok_or(AccountsError::TokenInvalid)?;
        let used = TokenUse {
            token_hash: token.hash(),
            purpose,
            at: self.now(),
        };
        let account = self.ports.email_tokens.consume(used).await?;
        let account = account.ok_or(AccountsError::TokenInvalid)?;
        let record = self.ports.accounts.get(account).await?;
        record.ok_or(AccountsError::TokenInvalid)
    }
}
