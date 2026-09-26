//! Admin passwords: Argon2id with H5's costs — 19,456 KiB, 2 passes, 1 lane —, stored as PHC
//! strings and computed on a blocking thread, never on the executor.

use std::fmt;

use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use thiserror::Error;

/// The memory of one hash, in KiB.
const ARGON2_MEMORY_KIB: u32 = 19_456;
/// The passes over that memory.
const ARGON2_PASSES: u32 = 2;
/// The lanes computed in parallel.
const ARGON2_LANES: u32 = 1;
/// The fewest characters of an admin's password.
pub const ADMIN_PASSWORD_MIN_CHARS: usize = 12;
/// The most characters of an admin's password.
pub const ADMIN_PASSWORD_MAX_CHARS: usize = 128;

/// Why a password could not be hashed or checked.
#[derive(Debug, Error)]
pub enum PasswordError {
    /// The password is shorter or longer than allowed.
    #[error("a password holds {ADMIN_PASSWORD_MIN_CHARS} to {ADMIN_PASSWORD_MAX_CHARS} characters")]
    Length,
    /// Argon2 or its worker thread failed.
    #[error("the password could not be hashed: {0}")]
    Hashing(String),
}

/// A password as typed, of 12 to 128 characters. Its `Debug` hides it.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    /// `text` as a password for a new admin: its length checked.
    ///
    /// # Errors
    ///
    /// [`PasswordError::Length`].
    pub fn new(text: String) -> Result<Self, PasswordError> {
        let length = text.chars().count();
        if !(ADMIN_PASSWORD_MIN_CHARS..=ADMIN_PASSWORD_MAX_CHARS).contains(&length) {
            return Err(PasswordError::Length);
        }
        Ok(Self(text))
    }

    /// `text` as sent to sign in: never checked for length, so that a wrong one costs the same.
    #[must_use]
    pub fn sent(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password(<redacted>)")
    }
}

/// The PHC string of `password`'s Argon2id hash, with a new random salt.
///
/// # Errors
///
/// [`PasswordError::Hashing`].
pub async fn hash(password: &Password) -> Result<String, PasswordError> {
    let password = password.clone();
    tokio::task::spawn_blocking(move || {
        argon2()?
            .hash_password(password.0.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| PasswordError::Hashing(error.to_string()))
    })
    .await
    .map_err(|error| PasswordError::Hashing(error.to_string()))?
}

/// Whether `password` is the one the PHC string `phc` was made from; a string that does not
/// parse matches nothing.
///
/// # Errors
///
/// [`PasswordError::Hashing`] when the worker thread fails.
pub async fn verify(password: &Password, phc: &str) -> Result<bool, PasswordError> {
    let (password, phc) = (password.clone(), phc.to_owned());
    tokio::task::spawn_blocking(move || {
        let Ok(hash) = argon2::PasswordHash::new(&phc) else {
            tracing::error!("a stored admin password hash does not parse");
            return false;
        };
        Argon2::default()
            .verify_password(password.0.as_bytes(), &hash)
            .is_ok()
    })
    .await
    .map_err(|error| PasswordError::Hashing(error.to_string()))
}

/// Spends the time of one hash on `password`, and forgets it: what an unknown address costs, so
/// that timing says nothing of which addresses are admins.
pub async fn spend(password: &Password) {
    let _forgotten = hash(password).await;
}

fn argon2() -> Result<Argon2<'static>, PasswordError> {
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_PASSES, ARGON2_LANES, None)
        .map_err(|error| PasswordError::Hashing(error.to_string()))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_hash_is_argon2id_and_matches_its_password_only() {
        let password = Password::new("correct horse battery".to_owned()).unwrap();
        let phc = hash(&password).await.unwrap();
        assert!(phc.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"), "{phc}");
        assert!(verify(&password, &phc).await.unwrap());
        let wrong = Password::sent("correct horse battery!".to_owned());
        assert!(!verify(&wrong, &phc).await.unwrap());
        assert!(!verify(&password, "$argon2id$broken").await.unwrap());
    }

    #[test]
    fn a_new_password_holds_twelve_to_128_characters() {
        assert!(Password::new("a".repeat(11)).is_err());
        assert!(Password::new("é".repeat(12)).is_ok());
        assert!(Password::new("a".repeat(129)).is_err());
        assert!(!format!("{:?}", Password::sent("secret".to_owned())).contains("secret"));
    }
}
