//! The TOTP secrets at rest: XChaCha20-Poly1305 under `LPA_TOTP_KEY`, a random 24-byte nonce
//! before the ciphertext, and the admin's id as associated data, so that a secret moved to
//! another admin's row no longer opens.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key as CipherKey, KeyInit, XChaCha20Poly1305, XNonce};
use uuid::Uuid;

use super::totp::TotpSecret;
use crate::config::Key;

/// The length of a nonce, in bytes.
const NONCE_BYTES: usize = 24;

/// Seals and opens the TOTP secrets.
#[derive(Clone)]
pub struct SecretBox {
    cipher: XChaCha20Poly1305,
}

impl SecretBox {
    /// The box of `key`.
    #[must_use]
    pub fn new(key: &Key) -> Self {
        let key = CipherKey::from(*key.as_bytes());
        Self {
            cipher: XChaCha20Poly1305::new(&key),
        }
    }

    /// `secret`, sealed for the admin `admin_id`: the nonce, then the ciphertext and its tag.
    #[must_use]
    pub fn seal(&self, admin_id: Uuid, secret: &TotpSecret) -> Vec<u8> {
        let nonce_bytes: [u8; NONCE_BYTES] = rand::random();
        let nonce = XNonce::from(nonce_bytes);
        let payload = Payload {
            msg: secret.as_bytes(),
            aad: admin_id.as_bytes(),
        };
        let sealed = self
            .cipher
            .encrypt(&nonce, payload)
            .unwrap_or_else(|_| unreachable!("a 20-byte secret is never too long"));
        [nonce_bytes.as_slice(), &sealed].concat()
    }

    /// The secret `sealed` holds for `admin_id`, or `None` when it was sealed under another key,
    /// for another admin, or altered.
    #[must_use]
    pub fn open(&self, admin_id: Uuid, sealed: &[u8]) -> Option<TotpSecret> {
        let (nonce, ciphertext) = sealed.split_at_checked(NONCE_BYTES)?;
        let nonce = XNonce::try_from(nonce).ok()?;
        let payload = Payload {
            msg: ciphertext,
            aad: admin_id.as_bytes(),
        };
        let secret = self.cipher.decrypt(&nonce, payload).ok()?;
        TotpSecret::from_bytes(&secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FromVariable;

    fn key(digit: char) -> Key {
        Key::from_variable(&digit.to_string().repeat(64)).unwrap()
    }

    #[test]
    fn a_sealed_secret_opens_only_for_its_admin_under_its_key() {
        let admin = Uuid::now_v7();
        let secret = TotpSecret::generate();
        let sealed = SecretBox::new(&key('a')).seal(admin, &secret);
        assert_eq!(sealed.len(), 24 + 20 + 16);
        assert_eq!(SecretBox::new(&key('a')).open(admin, &sealed), Some(secret));
        assert_eq!(SecretBox::new(&key('b')).open(admin, &sealed), None);
        assert_eq!(
            SecretBox::new(&key('a')).open(Uuid::now_v7(), &sealed),
            None
        );
        let mut altered = sealed.clone();
        altered[30] ^= 1;
        assert_eq!(SecretBox::new(&key('a')).open(admin, &altered), None);
        assert_eq!(SecretBox::new(&key('a')).open(admin, &sealed[..10]), None);
    }
}
