//! Password hashes: Argon2id, 19,456 KiB, 2 passes, 1 lane, stored as PHC strings and computed on
//! a blocking thread, never on the executor.

use std::fmt;

use argon2::{
    ARGON2ID_IDENT, Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
};

use super::AccountsError;
use super::values::Password;

/// The memory of one hash, in KiB.
pub const ARGON2_MEMORY_KIB: u32 = 19_456;
/// The passes over that memory.
pub const ARGON2_PASSES: u32 = 2;
/// The lanes computed in parallel.
pub const ARGON2_LANES: u32 = 1;

/// A password's hash as its PHC string, `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`. Its
/// `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// The hash of a PHC string, as a store read it back.
    #[must_use]
    pub fn from_phc(phc: String) -> Self {
        Self(phc)
    }

    /// The PHC string, for the store.
    #[must_use]
    pub fn as_phc(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PasswordHash(<redacted>)")
    }
}

/// What checking a password against its hash found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verification {
    /// The password is not the one hashed.
    Mismatch,
    /// The password is the one hashed; `needs_rehash` when the hash has other parameters than
    /// today's, and should be computed again while the password is at hand.
    Match {
        /// Whether the hash's parameters are not today's.
        needs_rehash: bool,
    },
}

/// How passwords are hashed: Argon2id with the parameters it is made with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordHashing {
    params: Params,
}

impl PasswordHashing {
    /// The parameters of docs/v1/accounts.md: 19,456 KiB, 2 passes, 1 lane.
    ///
    /// # Panics
    ///
    /// Never: the parameters are valid constants.
    #[must_use]
    #[allow(clippy::expect_used)] // Constants Argon2 accepts; a test checks them.
    pub fn standard() -> Self {
        Self::with_costs(ARGON2_MEMORY_KIB, ARGON2_PASSES, ARGON2_LANES)
            .expect("the standard Argon2id parameters are valid")
    }

    /// Hashing with other costs — memory in KiB, passes, lanes —, as a later tuning would;
    /// `None` when Argon2 refuses them.
    #[must_use]
    pub fn with_costs(memory_kib: u32, passes: u32, lanes: u32) -> Option<Self> {
        let params = Params::new(memory_kib, passes, lanes, None).ok()?;
        Some(Self { params })
    }

    /// The hash of `password`, with a new random salt.
    ///
    /// # Errors
    ///
    /// `service.unavailable` when the worker fails.
    pub async fn hash(&self, password: &Password) -> Result<PasswordHash, AccountsError> {
        let argon2 = self.argon2();
        let password = password.clone();
        let hashed = blocking(move || argon2.hash_password(password.as_bytes())).await?;
        let hash = hashed.map_err(|error| {
            tracing::error!(%error, "a password cannot be hashed");
            AccountsError::Unavailable
        })?;
        Ok(PasswordHash(hash.to_string()))
    }

    /// Spends the time of one hash on `password`, and forgets it: what an unknown address costs,
    /// so that timing says nothing of which addresses have an account.
    pub async fn spend(&self, password: &Password) {
        let _forgotten = self.hash(password).await;
    }

    /// Whether `password` is the one `hash` was made from, with the hash's own parameters.
    ///
    /// # Errors
    ///
    /// `service.unavailable` when the worker fails.
    pub async fn verify(
        &self,
        password: &Password,
        hash: &PasswordHash,
    ) -> Result<Verification, AccountsError> {
        let (password, phc, today) = (password.clone(), hash.0.clone(), self.params.clone());
        blocking(move || verify_phc(&password, &phc, &today)).await
    }

    fn argon2(&self) -> Argon2<'static> {
        Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone())
    }
}

/// Checks `password` against the PHC string `phc`; a string that does not parse is logged and
/// matches nothing.
fn verify_phc(password: &Password, phc: &str, today: &Params) -> Verification {
    let hash = match argon2::PasswordHash::new(phc) {
        Ok(hash) => hash,
        Err(error) => {
            tracing::error!(%error, "a stored password hash does not parse");
            return Verification::Mismatch;
        }
    };
    if Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_err()
    {
        return Verification::Mismatch;
    }
    Verification::Match {
        needs_rehash: !has_parameters(&hash, today),
    }
}

/// Whether `hash` is Argon2id, version 0x13, with the costs of `today`.
fn has_parameters(hash: &argon2::PasswordHash, today: &Params) -> bool {
    let is_current_version = hash.version == Some(Version::V0x13.into());
    let has_costs = Params::try_from(hash).is_ok_and(|params| {
        (params.m_cost(), params.t_cost(), params.p_cost())
            == (today.m_cost(), today.t_cost(), today.p_cost())
    });
    hash.algorithm == ARGON2ID_IDENT && is_current_version && has_costs
}

/// `work`'s output, computed on a blocking thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, AccountsError> {
    tokio::task::spawn_blocking(work).await.map_err(|error| {
        tracing::error!(%error, "the password worker failed");
        AccountsError::Unavailable
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password(text: &str) -> Password {
        Password::parse(text.to_owned()).unwrap()
    }

    #[tokio::test]
    async fn a_hash_is_an_argon2id_phc_string_with_the_standard_costs() {
        let hash = PasswordHashing::standard()
            .hash(&password("correct horse battery"))
            .await
            .unwrap();
        assert!(
            hash.as_phc().starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
            "{}",
            hash.as_phc()
        );
        assert!(!format!("{hash:?}").contains("argon2id"));
    }

    #[tokio::test]
    async fn the_hashed_password_matches_and_another_does_not() {
        let hashing = PasswordHashing::standard();
        let right = password("correct horse battery");
        let hash = hashing.hash(&right).await.unwrap();
        let matched = hashing.verify(&right, &hash).await.unwrap();
        let expected = Verification::Match {
            needs_rehash: false,
        };
        assert_eq!(matched, expected);
        let wrong = password("correct horse battery!");
        let other = hashing.verify(&wrong, &hash).await.unwrap();
        assert_eq!(other, Verification::Mismatch);
    }

    #[tokio::test]
    async fn a_hash_with_other_costs_matches_and_needs_a_rehash() {
        let older = PasswordHashing::with_costs(8_192, 1, 1).unwrap();
        let hash = older
            .hash(&password("correct horse battery"))
            .await
            .unwrap();
        let verified = PasswordHashing::standard()
            .verify(&password("correct horse battery"), &hash)
            .await
            .unwrap();
        assert_eq!(verified, Verification::Match { needs_rehash: true });
    }

    #[tokio::test]
    async fn a_hash_that_does_not_parse_matches_nothing() {
        let broken = PasswordHash::from_phc("$argon2id$broken".to_owned());
        let verified = PasswordHashing::standard()
            .verify(&password("correct horse battery"), &broken)
            .await
            .unwrap();
        assert_eq!(verified, Verification::Mismatch);
    }
}
