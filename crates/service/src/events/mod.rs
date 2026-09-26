//! Pseudonymizing an account for a product event (H13): a pure function, so that every adapter
//! that ever needs to correlate a subject computes it the same way, from the secret's bytes
//! alone.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::ids::AccountId;

/// Hexadecimal digits kept of the HMAC: 128 bits, enough to be collision-free, short enough to
/// stay a compact column.
const SUBJECT_HEX_DIGITS: usize = 32;

/// The pseudonymous subject of `account`: the first [`SUBJECT_HEX_DIGITS`] hexadecimal digits of
/// HMAC-SHA256(`secret`, the account's id) — stable for one account under one secret, and
/// meaningless without it (`docs/v1/server.md`, H13).
#[must_use]
pub fn subject(secret: &[u8], account: AccountId) -> String {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(secret)
        .unwrap_or_else(|_| unreachable!("HMAC accepts a key of any length"));
    mac.update(account.uuid().as_bytes());
    let digest = mac.finalize().into_bytes();
    digest
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0xf])
        .take(SUBJECT_HEX_DIGITS)
        .map(|nibble| char::from_digit(u32::from(nibble), 16).unwrap_or('0'))
        .collect()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn account(byte: u8) -> AccountId {
        AccountId::from_uuid(Uuid::from_bytes([byte; 16]))
    }

    #[test]
    fn the_subject_is_32_lowercase_hexadecimal_digits() {
        let value = subject(b"a secret", account(1));
        assert_eq!(value.len(), SUBJECT_HEX_DIGITS);
        assert!(value.bytes().all(|digit| digit.is_ascii_hexdigit()));
        assert_eq!(value, value.to_lowercase());
    }

    #[test]
    fn the_same_account_and_secret_give_the_same_subject() {
        assert_eq!(
            subject(b"a secret", account(1)),
            subject(b"a secret", account(1))
        );
    }

    #[test]
    fn another_account_gives_another_subject() {
        assert_ne!(
            subject(b"a secret", account(1)),
            subject(b"a secret", account(2))
        );
    }

    #[test]
    fn another_secret_gives_another_subject() {
        assert_ne!(
            subject(b"a secret", account(1)),
            subject(b"another secret", account(1))
        );
    }
}
