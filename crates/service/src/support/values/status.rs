//! Where a support request stands.

/// The status of a request: new → in progress → waiting for the user → resolved → closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SupportStatus {
    /// Nobody from the team has looked at it yet.
    New,
    /// The team is on it.
    InProgress,
    /// The team answered, and waits for the person.
    WaitingForUser,
    /// The team considers it solved.
    Resolved,
    /// Over: it takes no more replies.
    Closed,
}

impl SupportStatus {
    /// Every status, in the order a request goes through them.
    pub const ALL: [Self; 5] = [
        Self::New,
        Self::InProgress,
        Self::WaitingForUser,
        Self::Resolved,
        Self::Closed,
    ];

    /// The status's value in storage and in the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::InProgress => "in_progress",
            Self::WaitingForUser => "waiting_for_user",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }

    /// The status of `text`, as [`SupportStatus::as_str`] writes it.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|status| status.as_str() == text)
    }

    /// The status once the person replies, or `None` when the request takes no reply: a reply
    /// moves `waiting_for_user` back to `in_progress`, a closed request refuses it, and the
    /// others keep their status.
    #[must_use]
    pub const fn after_user_reply(self) -> Option<Self> {
        match self {
            Self::Closed => None,
            Self::WaitingForUser => Some(Self::InProgress),
            Self::New | Self::InProgress | Self::Resolved => Some(self),
        }
    }
}
