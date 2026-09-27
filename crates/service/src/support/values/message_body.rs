//! The text of a support message.

use life_pixel_core::limits::SUPPORT_MESSAGE_MAX_CHARS;

use crate::support::SupportError;

/// The fewest characters of a message, once trimmed.
pub const MESSAGE_MIN_CHARS: usize = 1;

/// A message's text, trimmed, of [`MESSAGE_MIN_CHARS`] to `SUPPORT_MESSAGE_MAX_CHARS`
/// characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageBody(String);

impl MessageBody {
    /// The body of `text`, without its surrounding whitespace.
    ///
    /// # Errors
    ///
    /// `support.message_length`, with `min` and `max`, when it is empty or too long.
    pub fn parse(text: &str) -> Result<Self, SupportError> {
        let trimmed = text.trim();
        let length = trimmed.chars().count();
        if !(MESSAGE_MIN_CHARS..=SUPPORT_MESSAGE_MAX_CHARS).contains(&length) {
            return Err(SupportError::MessageLength);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The text, owned.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}
