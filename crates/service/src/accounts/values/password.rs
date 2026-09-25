//! A password as a person types it: a length, no other rule.

use std::fmt;

use life_pixel_core::limits::{PASSWORD_MAX_CHARS, PASSWORD_MIN_CHARS};

use crate::accounts::AccountsError;

/// A password of `PASSWORD_MIN_CHARS` to `PASSWORD_MAX_CHARS` characters, kept as typed: never
/// trimmed. Its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    /// The password `text`.
    ///
    /// # Errors
    ///
    /// `auth.password_length`, with `min` and `max`, when it is too short or too long.
    pub fn parse(text: String) -> Result<Self, AccountsError> {
        let length = text.chars().count();
        if !(PASSWORD_MIN_CHARS..=PASSWORD_MAX_CHARS).contains(&length) {
            return Err(AccountsError::PasswordLength);
        }
        Ok(Self(text))
    }

    /// The password's bytes, for the hash alone.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_holds_min_to_max_characters_whatever_they_are() {
        let shortest = "é".repeat(PASSWORD_MIN_CHARS);
        let longest = " ".repeat(PASSWORD_MAX_CHARS);
        assert!(Password::parse(shortest).is_ok());
        assert!(Password::parse(longest).is_ok());
        for text in [
            "é".repeat(PASSWORD_MIN_CHARS - 1),
            "a".repeat(PASSWORD_MAX_CHARS + 1),
        ] {
            assert_eq!(Password::parse(text), Err(AccountsError::PasswordLength));
        }
    }

    #[test]
    fn a_password_never_reaches_a_log() {
        let password = Password::parse("correct horse battery".to_owned()).unwrap();
        assert!(!format!("{password:?}").contains("horse"));
    }
}
