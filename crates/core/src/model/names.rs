use std::fmt;

use serde::Serialize;

use crate::error::DocumentError;
use crate::limits::{NAME_MAX_CHARS, TAG_NAME_MAX_CHARS};

/// The fewest characters of a name or a tag name.
const MIN_CHARS: usize = 1;

/// A title, project name or layer name: 1 to [`NAME_MAX_CHARS`] characters after trimming, no
/// control character.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Name(String);

impl Name {
    /// The name `text`, trimmed.
    ///
    /// # Errors
    ///
    /// [`DocumentError::Name`] when the trimmed text is empty, too long or holds a control
    /// character.
    pub fn new(text: &str) -> Result<Self, DocumentError> {
        let trimmed = text.trim();
        let length = trimmed.chars().count();
        let is_valid = (MIN_CHARS..=NAME_MAX_CHARS).contains(&length)
            && !trimmed.chars().any(char::is_control);
        is_valid
            .then(|| Self(trimmed.to_owned()))
            .ok_or(DocumentError::Name)
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A tag name: 1 to [`TAG_NAME_MAX_CHARS`] characters among `a`–`z`, `0`–`9`, `-` and `_`,
/// starting with a letter. Uniqueness is the animation's rule.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct TagName(String);

impl TagName {
    /// The tag name `text`, as given.
    ///
    /// # Errors
    ///
    /// [`DocumentError::Tag`], naming `text`, when it breaks a rule of tag names.
    pub fn new(text: &str) -> Result<Self, DocumentError> {
        let length = text.chars().count();
        let starts_with_letter = text.starts_with(|first: char| first.is_ascii_lowercase());
        let is_valid = (MIN_CHARS..=TAG_NAME_MAX_CHARS).contains(&length)
            && starts_with_letter
            && text.chars().all(is_tag_name_char);
        is_valid
            .then(|| Self(text.to_owned()))
            .ok_or_else(|| DocumentError::Tag {
                name: text.to_owned(),
            })
    }

    /// The tag name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn is_tag_name_char(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed_and_bounded() {
        assert_eq!(Name::new("  Mascot ").unwrap().as_str(), "Mascot");
        assert_eq!(
            Name::new(&"é".repeat(NAME_MAX_CHARS))
                .unwrap()
                .as_str()
                .chars()
                .count(),
            100
        );
        for text in [
            "",
            "   ",
            &"a".repeat(NAME_MAX_CHARS + 1),
            "tab\there",
            "bell\u{7}",
        ] {
            assert_eq!(Name::new(text), Err(DocumentError::Name), "{text:?}");
        }
    }

    #[test]
    fn a_tag_name_follows_its_alphabet() {
        for text in [
            "idle",
            "walk-left",
            "run_2",
            &"a".repeat(TAG_NAME_MAX_CHARS),
        ] {
            assert!(TagName::new(text).is_ok(), "{text:?}");
        }
        for text in ["", "2run", "-idle", "Idle", "walk left", &"a".repeat(33)] {
            let error = DocumentError::Tag {
                name: text.to_owned(),
            };
            assert_eq!(TagName::new(text), Err(error), "{text:?}");
        }
    }
}
