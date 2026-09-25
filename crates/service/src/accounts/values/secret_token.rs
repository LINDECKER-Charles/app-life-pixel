//! The secrets handed to one client — a session's cookie, the token of an emailed link — and the
//! hashes the stores keep instead.

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// The random bytes of a secret token.
pub const SECRET_TOKEN_BYTES: usize = 32;
/// The characters of a token written base64url without padding: 32 bytes make 43.
const SECRET_TOKEN_CHARS: usize = 43;
/// The bytes of a token's hash: SHA-256.
pub const TOKEN_HASH_BYTES: usize = 32;

/// A secret: 32 random bytes written base64url without padding. Only its [`TokenHash`] is
/// stored, so that a copy of the database opens no session. Its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretToken(String);

impl SecretToken {
    /// A new token, from the thread's cryptographically secure generator.
    #[must_use]
    pub fn generate() -> Self {
        let bytes: [u8; SECRET_TOKEN_BYTES] = rand::random();
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    /// The token a client sent, when it has the shape of one; `None` otherwise, which no stored
    /// hash could match.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let is_token = text.len() == SECRET_TOKEN_CHARS
            && text
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        is_token.then(|| Self(text.to_owned()))
    }

    /// The token, for the client alone: a cookie or a link.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// What the stores keep: the SHA-256 of the token's text.
    #[must_use]
    pub fn hash(&self) -> TokenHash {
        TokenHash(Sha256::digest(self.0.as_bytes()).into())
    }
}

impl fmt::Debug for SecretToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretToken(<redacted>)")
    }
}

/// The SHA-256 of a [`SecretToken`]: the key of a session or of an email token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenHash([u8; TOKEN_HASH_BYTES]);

impl TokenHash {
    /// The hash of these bytes, as a store read them back.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; TOKEN_HASH_BYTES]) -> Self {
        Self(bytes)
    }

    /// The hash's bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; TOKEN_HASH_BYTES] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_43_base64url_characters_and_parses_back() {
        let token = SecretToken::generate();
        assert_eq!(token.expose().len(), SECRET_TOKEN_CHARS);
        let parsed = SecretToken::parse(token.expose()).unwrap();
        assert_eq!(parsed.hash(), token.hash());
        assert_ne!(SecretToken::generate().hash(), token.hash());
    }

    #[test]
    fn a_text_without_the_shape_of_a_token_is_none() {
        let token = SecretToken::generate();
        let longer = format!("{}A", token.expose());
        let padded = format!("{}=", &token.expose()[1..]);
        for text in ["", "short", longer.as_str(), padded.as_str()] {
            assert!(SecretToken::parse(text).is_none(), "{text:?}");
        }
    }

    #[test]
    fn a_token_never_reaches_a_log() {
        let token = SecretToken::generate();
        assert!(!format!("{token:?}").contains(token.expose()));
    }
}
