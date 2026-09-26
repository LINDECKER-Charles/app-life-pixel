//! A support request's id.

use std::fmt;

use uuid::Uuid;

/// A support request's id: a UUID that is never taken for another kind of id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SupportRequestId(Uuid);

impl SupportRequestId {
    /// The id of `uuid`.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// The UUID of this id.
    #[must_use]
    pub const fn uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for SupportRequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
