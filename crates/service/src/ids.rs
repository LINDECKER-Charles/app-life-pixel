//! The ids of the service's entities: UUID newtypes, so that one kind of id is never passed for
//! another.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
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

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

uuid_id!(
    /// A project's id.
    ProjectId
);
uuid_id!(
    /// An animation's id.
    AnimationId
);
uuid_id!(
    /// A hosted account's id.
    AccountId
);
