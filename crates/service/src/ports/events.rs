//! Product events: pseudonymous, first-party, and only from the hosted service.

use crate::ids::AccountId;

/// The size classes' upper bounds, in bytes, each with its class; above the last, `ge_1m`.
const SIZE_CLASSES: [(u64, &str); 4] = [
    (1 << 10, "lt_1k"),
    (10 << 10, "lt_10k"),
    (100 << 10, "lt_100k"),
    (1 << 20, "lt_1m"),
];
/// The class of a size at or above 1 MiB.
const LARGEST_SIZE_CLASS: &str = "ge_1m";

/// A pseudonymous product event: a name, the account it concerns, flat properties.
///
/// Events are data rather than an enum, so that each task adds its own without touching a shared
/// type; their names and properties are fixed by the design, and nowhere else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductEvent {
    /// The event's name, `document_saved` for example.
    pub name: &'static str,
    /// The account it concerns, if any.
    pub account: Option<AccountId>,
    /// Its properties, flat.
    pub properties: Vec<(&'static str, String)>,
}

impl ProductEvent {
    /// The `size` property's class of `bytes`: `lt_1k`, `lt_10k`, `lt_100k`, `lt_1m` or `ge_1m`,
    /// in binary units.
    #[must_use]
    pub fn size_class(bytes: u64) -> &'static str {
        SIZE_CLASSES
            .iter()
            .find(|(bound, _)| bytes < *bound)
            .map_or(LARGEST_SIZE_CLASS, |(_, class)| class)
    }
}

/// Records a product event; never blocks and never fails the caller.
pub trait EventSink: Send + Sync {
    /// Records `event`.
    fn record(&self, event: ProductEvent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_fall_in_their_class() {
        let classes = [
            (0, "lt_1k"),
            (1_023, "lt_1k"),
            (1_024, "lt_10k"),
            (10_239, "lt_10k"),
            (10_240, "lt_100k"),
            (102_400, "lt_1m"),
            (1_048_575, "lt_1m"),
            (1_048_576, "ge_1m"),
        ];
        for (bytes, class) in classes {
            assert_eq!(ProductEvent::size_class(bytes), class, "{bytes}");
        }
    }
}
