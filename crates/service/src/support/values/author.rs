//! Who wrote a message of a support request.

/// The author of a message: the person who asked, or the support team.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Author {
    /// The account that opened the request.
    User,
    /// An admin of the support team.
    Team,
}

impl Author {
    /// The author's value in storage and in the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Team => "team",
        }
    }

    /// The author of `text`, as [`Author::as_str`] writes it.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        [Self::User, Self::Team]
            .into_iter()
            .find(|author| author.as_str() == text)
    }
}
