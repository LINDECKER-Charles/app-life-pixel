//! `create-admin <email>` and `disable-admin <email>`, run on the admin server's host: the only
//! ways an admin account comes and goes. `create-admin` reads the password twice without echo —
//! once from standard input with `--password-stdin` —, and prints the `otpauth://` URI to enrol
//! the TOTP secret in an authenticator app; nothing else ever shows the secret.

use std::fmt::Write as _;
use std::io::{self, BufRead};
use std::sync::Arc;

use sqlx::PgPool;
use thiserror::Error;

use crate::admins::clock::SystemClock;
use crate::admins::password::{Password, PasswordError};
use crate::admins::secret_box::SecretBox;
use crate::admins::store::{AdminIdentity, AdminStore};
use crate::admins::totp::TotpSecret;
use crate::admins::{Admins, CreateAdminError};
use crate::config::AccountsConfig;
use crate::database::{self, DatabaseError};

/// The issuer an authenticator app shows, percent-encoded.
const ISSUER: &str = "Life%20Pixel";

/// Why an admin could not be created or disabled.
#[derive(Debug, Error)]
pub enum AdminCommandError {
    /// The password could not be read.
    #[error("the password cannot be read: {0}")]
    Read(#[from] io::Error),
    /// The two passwords typed differ.
    #[error("the passwords differ")]
    Mismatch,
    /// The password is too short or too long.
    #[error(transparent)]
    Password(#[from] PasswordError),
    /// The database does not answer, or cannot be migrated.
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// The admin could not be created.
    #[error(transparent)]
    Create(#[from] CreateAdminError),
    /// The admin could not be disabled.
    #[error("the admin database: {0}")]
    Disable(#[from] sqlx::Error),
}

/// Where `create-admin` reads the password.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordInput {
    /// From the terminal, twice, without echo.
    Prompt,
    /// The first line of standard input, for scripts.
    Stdin,
}

/// The password, read from `input`.
///
/// # Errors
///
/// When it cannot be read, the two typed differ, or its length is out of bounds.
pub fn read_password(input: PasswordInput) -> Result<Password, AdminCommandError> {
    let text = match input {
        PasswordInput::Stdin => {
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line)?;
            line.trim_end_matches(['\n', '\r']).to_owned()
        }
        PasswordInput::Prompt => {
            let first = rpassword::prompt_password("Password: ")?;
            let second = rpassword::prompt_password("Password again: ")?;
            if first != second {
                return Err(AdminCommandError::Mismatch);
            }
            first
        }
    };
    Ok(Password::new(text)?)
}

/// Creates the admin `email` with `password` and a new TOTP secret, after migrating the
/// database: the admin, and the URI to enrol the secret.
///
/// # Errors
///
/// When the database fails, the address is invalid or taken, or the password cannot be hashed.
pub async fn create_admin(
    config: &AccountsConfig,
    email: &str,
    password: &Password,
) -> Result<(AdminIdentity, String), AdminCommandError> {
    let pool = open(config).await?;
    let (admin, secret) = admins(config, pool).create(email, password).await?;
    let uri = otpauth_uri(&admin.email, &secret);
    Ok((admin, uri))
}

/// Disables the admin `email` and ends its sessions: `false` when there is no such admin.
///
/// # Errors
///
/// When the database fails.
pub async fn disable_admin(
    config: &AccountsConfig,
    email: &str,
) -> Result<bool, AdminCommandError> {
    let pool = open(config).await?;
    Ok(admins(config, pool).disable(email).await?)
}

async fn open(config: &AccountsConfig) -> Result<PgPool, DatabaseError> {
    let pool = database::connect(&config.database_url).await?;
    database::migrate(&pool).await?;
    Ok(pool)
}

fn admins(config: &AccountsConfig, pool: PgPool) -> Admins {
    let secrets = SecretBox::new(&config.totp_key);
    Admins::new(AdminStore::new(pool), secrets, Arc::new(SystemClock))
}

/// `otpauth://totp/Life%20Pixel:<email>?secret=<base32>&issuer=Life%20Pixel`: SHA-1, 6 digits
/// and 30 seconds, the defaults an authenticator app assumes.
#[must_use]
pub fn otpauth_uri(email: &str, secret: &TotpSecret) -> String {
    let label = format!("{ISSUER}:{}", percent_encoded(email));
    let secret = secret.to_base32();
    format!("otpauth://totp/{label}?secret={secret}&issuer={ISSUER}")
}

/// `text` with everything but RFC 3986's unreserved characters percent-encoded.
fn percent_encoded(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _infallible = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uri_names_the_issuer_the_admin_and_the_secret() {
        let secret = TotpSecret::from_bytes(b"12345678901234567890").unwrap();
        assert_eq!(
            otpauth_uri("ada+ops@example.org", &secret),
            "otpauth://totp/Life%20Pixel:ada%2Bops%40example.org\
             ?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Life%20Pixel"
        );
    }
}
