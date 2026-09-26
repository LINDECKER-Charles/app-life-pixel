//! What a support request is about.

use crate::support::SupportError;

/// The category a person picks for a request (docs/admin-console.md, "Support requests").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Something does not work.
    Bug,
    /// Signing in, the address, the account itself.
    Account,
    /// Plans and payments.
    Billing,
    /// Data and privacy: access and erasure requests, with their legal deadline.
    DataProtection,
    /// Content or behaviour to report.
    Abuse,
    /// Anything else.
    Other,
}

impl Category {
    /// Every category, in the order the app offers them.
    pub const ALL: [Self; 6] = [
        Self::Bug,
        Self::Account,
        Self::Billing,
        Self::DataProtection,
        Self::Abuse,
        Self::Other,
    ];

    /// The category's value in storage, in the API and in product events.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Account => "account",
            Self::Billing => "billing",
            Self::DataProtection => "data_protection",
            Self::Abuse => "abuse",
            Self::Other => "other",
        }
    }

    /// The category of `text`, as [`Category::as_str`] writes it.
    ///
    /// # Errors
    ///
    /// `support.category` for any other text.
    pub fn parse(text: &str) -> Result<Self, SupportError> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == text)
            .ok_or(SupportError::Category)
    }
}
