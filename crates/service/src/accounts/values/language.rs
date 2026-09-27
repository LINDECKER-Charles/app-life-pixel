//! The language of an account's emails and interface: one of the catalogues'.

use crate::accounts::AccountsError;

/// A language code among those the server has a catalogue for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Language(String);

impl Language {
    /// The language `code`, when `available` lists it.
    ///
    /// # Errors
    ///
    /// `account.language`, with the codes `available`, when it does not.
    pub fn parse(code: &str, available: &[String]) -> Result<Self, AccountsError> {
        if !available.iter().any(|known| known == code) {
            return Err(AccountsError::Language {
                available: available.to_vec(),
            });
        }
        Ok(Self(code.to_owned()))
    }

    /// The code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_is_one_of_the_catalogues() {
        let available = ["en".to_owned(), "fr".to_owned()];
        assert_eq!(Language::parse("fr", &available).unwrap().as_str(), "fr");
        assert_eq!(
            Language::parse("de", &available),
            Err(AccountsError::Language {
                available: available.to_vec()
            })
        );
    }
}
