//! An account's email address, checked the same way at sign-up, sign-in and password reset.

use std::fmt;

use life_pixel_core::limits::EMAIL_MAX_CHARS;

use crate::accounts::AccountsError;

/// The separator of an address's local part and domain.
const AT: char = '@';
/// What the domain of an address holds at least once.
const DOT: char = '.';

/// An email address: trimmed, at most `EMAIL_MAX_CHARS` characters, one `@` with text on both
/// sides, a dot in the domain, and no space or control character. Its case is kept; the stores
/// compare addresses whatever their case.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// The address `text` holds, trimmed.
    ///
    /// # Errors
    ///
    /// `auth.email_invalid` when it breaks a rule.
    pub fn parse(text: &str) -> Result<Self, AccountsError> {
        let address = text.trim();
        let is_short_enough = address.chars().count() <= EMAIL_MAX_CHARS;
        let is_plain = !address
            .chars()
            .any(|character| character.is_whitespace() || character.is_control());
        let is_shaped = match address.split_once(AT) {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.is_empty()
                    && !domain.contains(AT)
                    && domain.contains(DOT)
            }
            None => false,
        };
        if !(is_short_enough && is_plain && is_shaped) {
            return Err(AccountsError::EmailInvalid);
        }
        Ok(Self(address.to_owned()))
    }

    /// The address.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_is_trimmed_and_keeps_its_case() {
        let address = EmailAddress::parse("  Ada.Lovelace@Example.org \n").unwrap();
        assert_eq!(address.as_str(), "Ada.Lovelace@Example.org");
    }

    #[test]
    fn an_address_needs_one_at_with_text_around_and_a_dotted_domain() {
        for text in [
            "",
            "ada",
            "@example.org",
            "ada@",
            "ada@example",
            "ada@@example.org",
            "ada@exa@mple.org",
            "ada lovelace@example.org",
            "ada@example.org\r\nBcc: eve@example.org",
        ] {
            assert_eq!(
                EmailAddress::parse(text),
                Err(AccountsError::EmailInvalid),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_address_holds_at_most_email_max_chars_characters() {
        let domain = "@example.org";
        let local = "é".repeat(EMAIL_MAX_CHARS - domain.len());
        assert!(EmailAddress::parse(&format!("{local}{domain}")).is_ok());
        let longer = format!("a{local}{domain}");
        assert_eq!(
            EmailAddress::parse(&longer),
            Err(AccountsError::EmailInvalid)
        );
    }
}
