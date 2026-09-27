//! `LPA_ROOT_ADMIN_EMAIL`, `LPA_ROOT_ADMIN_PASSWORD` and `LPA_ROOT_ADMIN_TOTP_SECRET`: the root
//! admin `serve` creates on a host that has no admin at all, so that a new host can be signed in
//! to without a shell on it. The three go together — all set, or none —, so that a half-written
//! block stops the start instead of being silently ignored. The TOTP secret comes from the
//! operator, who enrols it in an authenticator: the server never prints a secret.

use super::env::{ConfigError, Env, FromVariable};
use crate::admins::password::{ADMIN_PASSWORD_MAX_CHARS, ADMIN_PASSWORD_MIN_CHARS, Password};
use crate::admins::sign_in::{AdminAccount, valid_email};
use crate::admins::totp::TotpSecret;

const EMAIL: &str = "LPA_ROOT_ADMIN_EMAIL";
const PASSWORD: &str = "LPA_ROOT_ADMIN_PASSWORD";
const TOTP_SECRET: &str = "LPA_ROOT_ADMIN_TOTP_SECRET";
const EMAIL_EXPECTED: &str = "an email address";
const PASSWORD_EXPECTED: &str = "12 to 128 characters";
// The bounds PASSWORD_EXPECTED states are those Password::new checks.
const _: () = assert!(ADMIN_PASSWORD_MIN_CHARS == 12 && ADMIN_PASSWORD_MAX_CHARS == 128);

/// The root admin, or `None` when none of the three variables is set. The password is trimmed,
/// as every value is.
///
/// # Errors
///
/// When one is set and another is missing, or one is invalid.
pub fn read_root_admin(env: &Env<'_>) -> Result<Option<AdminAccount>, ConfigError> {
    let variables = [EMAIL, PASSWORD, TOTP_SECRET];
    if variables
        .iter()
        .all(|variable| env.optional(variable).is_none())
    {
        return Ok(None);
    }
    let email: String = env.parse(EMAIL)?;
    let email = valid_email(&email).ok_or(ConfigError::Invalid {
        variable: EMAIL,
        expected: EMAIL_EXPECTED,
    })?;
    Ok(Some(AdminAccount {
        email,
        password: env.parse(PASSWORD)?,
        secret: env.parse(TOTP_SECRET)?,
    }))
}

impl FromVariable for Password {
    const EXPECTED: &'static str = PASSWORD_EXPECTED;

    fn from_variable(value: &str) -> Option<Self> {
        Self::new(value.to_owned()).ok()
    }
}

impl FromVariable for TotpSecret {
    const EXPECTED: &'static str =
        "20 random bytes in base32, 32 characters, such as openssl rand 20 | base32";

    fn from_variable(value: &str) -> Option<Self> {
        Self::from_base32(value)
    }
}
