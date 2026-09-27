//! Signing in: an address, a password and a TOTP code, all three right or
//! `admin.invalid_credentials`, whichever is wrong; and creating an admin, for `create-admin`.

use thiserror::Error;
use uuid::Uuid;

use super::password::{self, Password, PasswordError};
use super::sessions::{SessionTimes, SessionToken};
use super::store::{AdminIdentity, AdminRecord, NewAdmin, NewSession};
use super::totp::TotpSecret;
use super::{Admins, EMAIL_MAX_CHARS};

/// What a sign-in sends.
#[derive(Clone, Debug)]
pub struct Credentials {
    /// The address.
    pub email: String,
    /// The password.
    pub password: Password,
    /// The TOTP code.
    pub code: String,
}

/// A session just opened.
#[derive(Clone, Debug)]
pub struct SignedIn {
    /// The cookie's token.
    pub token: SessionToken,
    /// The admin.
    pub admin: AdminIdentity,
}

/// Why a sign-in failed.
#[derive(Debug, Error)]
pub enum SignInError {
    /// The address, the password or the code is wrong, or the admin is disabled.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// The database or the password worker failed.
    #[error("the admin accounts are unavailable: {0}")]
    Unavailable(String),
}

impl From<sqlx::Error> for SignInError {
    fn from(error: sqlx::Error) -> Self {
        Self::Unavailable(error.to_string())
    }
}

impl From<PasswordError> for SignInError {
    fn from(error: PasswordError) -> Self {
        Self::Unavailable(error.to_string())
    }
}

/// Why an admin could not be created.
#[derive(Debug, Error)]
pub enum CreateAdminError {
    /// The address does not look like one.
    #[error("the address is not a valid email address")]
    Email,
    /// Another admin has the address.
    #[error("an admin with this address exists")]
    Taken,
    /// The password is too short or too long, or could not be hashed.
    #[error(transparent)]
    Password(#[from] PasswordError),
    /// The database failed.
    #[error("the admin database: {0}")]
    Database(#[from] sqlx::Error),
}

impl Admins {
    /// Opens a session for the admin whose address, password and current TOTP code `credentials`
    /// holds; the code's step is used up.
    ///
    /// # Errors
    ///
    /// [`SignInError::InvalidCredentials`] for any wrong part, [`SignInError::Unavailable`].
    pub async fn sign_in(&self, credentials: &Credentials) -> Result<SignedIn, SignInError> {
        let email = credentials.email.trim();
        let found = if email.chars().count() <= EMAIL_MAX_CHARS {
            self.store.find_active(email).await?
        } else {
            None
        };
        let Some(admin) = found else {
            password::spend(&credentials.password).await;
            return Err(SignInError::InvalidCredentials);
        };
        if !password::verify(&credentials.password, &admin.password_hash).await? {
            return Err(SignInError::InvalidCredentials);
        }
        self.use_code(&admin, &credentials.code).await?;
        let identity = AdminIdentity {
            id: admin.id,
            email: admin.email,
        };
        self.open_session(identity).await
    }

    /// A new session of `admin`.
    async fn open_session(&self, admin: AdminIdentity) -> Result<SignedIn, SignInError> {
        let token = SessionToken::generate();
        let session = NewSession {
            hash: token.hash(),
            admin_id: admin.id,
            times: SessionTimes::starting(self.clock.now()),
        };
        self.store.insert_session(&session).await?;
        Ok(SignedIn { token, admin })
    }

    /// Checks `code` against the admin's secret, and uses its step up.
    async fn use_code(&self, admin: &AdminRecord, code: &str) -> Result<(), SignInError> {
        let Some(secret) = self.secrets.open(admin.id, &admin.totp_secret) else {
            tracing::error!(admin = %admin.id, "an admin's TOTP secret does not open: LPA_TOTP_KEY?");
            return Err(SignInError::InvalidCredentials);
        };
        let now = self.clock.now();
        let step = secret
            .verify(code.trim(), (now, admin.totp_last_step))
            .ok_or(SignInError::InvalidCredentials)?;
        if !self.store.claim_totp_step(admin.id, step).await? {
            return Err(SignInError::InvalidCredentials);
        }
        Ok(())
    }

    /// Creates an admin with `email` and `password`, and a new TOTP secret, which it returns for
    /// the authenticator.
    ///
    /// # Errors
    ///
    /// When the address is invalid or taken, the password is refused, or the database fails.
    pub async fn create(
        &self,
        email: &str,
        password: &Password,
    ) -> Result<(AdminIdentity, TotpSecret), CreateAdminError> {
        let email = valid_email(email).ok_or(CreateAdminError::Email)?;
        let id = Uuid::now_v7();
        let secret = TotpSecret::generate();
        let admin = NewAdmin {
            id,
            email: email.clone(),
            password_hash: password::hash(password).await?,
            totp_secret: self.secrets.seal(id, &secret),
            created_at: self.clock.now(),
        };
        if !self.store.insert(&admin).await? {
            return Err(CreateAdminError::Taken);
        }
        Ok((AdminIdentity { id, email }, secret))
    }
}

/// `text` trimmed, when it looks like an email address: one `@` between a local part and a
/// domain, no space, 254 characters at most.
#[must_use]
pub fn valid_email(text: &str) -> Option<String> {
    let email = text.trim();
    let (local, domain) = email.split_once('@')?;
    let is_valid = !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && email.chars().count() <= EMAIL_MAX_CHARS
        && !email
            .chars()
            .any(|character| character.is_whitespace() || character.is_control());
    is_valid.then(|| email.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_admin_address_is_checked_for_its_shape() {
        assert_eq!(
            valid_email(" ada@example.org ").as_deref(),
            Some("ada@example.org")
        );
        for invalid in [
            "ada",
            "@example.org",
            "ada@org",
            "ada@x@y.org",
            "a da@x.org",
        ] {
            assert_eq!(valid_email(invalid), None, "{invalid}");
        }
        assert_eq!(valid_email(&format!("{}@x.org", "a".repeat(250))), None);
    }
}
