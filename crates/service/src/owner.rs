//! Whose library a call reads or changes.

use crate::ids::AccountId;

/// The owner of projects and animations. An owner never sees another owner's data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    /// A hosted account: the free plan's quota applies.
    Account(AccountId),
    /// The local library of the desktop app or the CLI: no quota, no telemetry.
    Local,
}

impl Owner {
    /// The account this owner is, if any.
    #[must_use]
    pub fn account(&self) -> Option<AccountId> {
        match self {
            Self::Account(account) => Some(*account),
            Self::Local => None,
        }
    }
}
