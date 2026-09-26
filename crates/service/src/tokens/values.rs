//! A token's secret and its scopes.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::accounts::{SecretToken, TokenHash};

/// What every personal access token starts with, so that a leaked one is recognizable.
pub const ACCESS_TOKEN_PREFIX: &str = "lp_pat_";
/// The characters of the random part a token's displayed prefix keeps.
const DISPLAYED_RANDOM_CHARS: usize = 4;

/// What a token grants: each MCP tool needs one scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenScope {
    /// Reading animations, their previews and their snippets.
    Read,
    /// Creating and changing animations.
    Write,
    /// Exporting animations as files.
    Export,
}

impl TokenScope {
    /// Every scope, in the order they are listed.
    pub const ALL: [Self; 3] = [Self::Read, Self::Write, Self::Export];

    /// The scope's name: `read`, `write` or `export`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Export => "export",
        }
    }

    /// The scope named `name`, if any.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scope| scope.as_str() == name)
    }
}

/// A personal access token: `lp_pat_` and 32 random bytes in base64url. Only its
/// [`TokenHash`] is stored; its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct AccessTokenSecret(String);

impl AccessTokenSecret {
    /// A new token, from the thread's cryptographically secure generator.
    #[must_use]
    pub fn generate() -> Self {
        let random = SecretToken::generate();
        Self(format!("{ACCESS_TOKEN_PREFIX}{}", random.expose()))
    }

    /// The token a client sent, when it has the shape of one; `None` otherwise, which no stored
    /// hash could match.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let random = text.strip_prefix(ACCESS_TOKEN_PREFIX)?;
        SecretToken::parse(random).map(|_| Self(text.to_owned()))
    }

    /// The token, for its owner alone, once.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// What the store keeps: the SHA-256 of the token's text.
    #[must_use]
    pub fn hash(&self) -> TokenHash {
        TokenHash::from_bytes(Sha256::digest(self.0.as_bytes()).into())
    }

    /// What the owner sees of it later: `lp_pat_` and the first 4 random characters.
    #[must_use]
    pub fn displayed_prefix(&self) -> String {
        let end = ACCESS_TOKEN_PREFIX.len() + DISPLAYED_RANDOM_CHARS;
        self.0[..end].to_owned()
    }
}

impl fmt::Debug for AccessTokenSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessTokenSecret(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_the_prefix_and_43_base64url_characters_and_parses_back() {
        let token = AccessTokenSecret::generate();
        assert!(token.expose().starts_with("lp_pat_"));
        assert_eq!(token.expose().len(), 7 + 43);
        let parsed = AccessTokenSecret::parse(token.expose()).unwrap();
        assert_eq!(parsed.hash(), token.hash());
        assert_eq!(token.displayed_prefix(), token.expose()[..11]);
        assert_ne!(AccessTokenSecret::generate().hash(), token.hash());
    }

    #[test]
    fn a_text_without_the_shape_of_a_token_is_none() {
        let token = AccessTokenSecret::generate();
        let random = &token.expose()[7..];
        let other_prefix = format!("lp_xxx_{random}");
        let shorter = &token.expose()[..40];
        for text in ["", "lp_pat_", random, other_prefix.as_str(), shorter] {
            assert!(AccessTokenSecret::parse(text).is_none(), "{text:?}");
        }
    }

    #[test]
    fn a_token_never_reaches_a_log() {
        let token = AccessTokenSecret::generate();
        assert!(!format!("{token:?}").contains(token.expose()));
    }

    #[test]
    fn scopes_parse_from_their_names() {
        for scope in TokenScope::ALL {
            assert_eq!(TokenScope::parse(scope.as_str()), Some(scope));
        }
        assert_eq!(TokenScope::parse("admin"), None);
    }
}
