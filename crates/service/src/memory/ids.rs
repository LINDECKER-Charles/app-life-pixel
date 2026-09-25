//! Ids that count up, so that tests can name them.

use std::sync::atomic::{AtomicU64, Ordering};

use uuid::Uuid;

use crate::ports::IdGenerator;

/// Ids 1, 2, 3… as UUIDs: `SequentialIds::id(n)` is the `n`-th id made.
#[derive(Debug, Default)]
pub struct SequentialIds {
    made: AtomicU64,
}

impl SequentialIds {
    /// A generator whose first id is `SequentialIds::id(1)`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The `n`-th id this generator makes.
    #[must_use]
    pub fn id(n: u64) -> Uuid {
        Uuid::from_u128(u128::from(n))
    }
}

impl IdGenerator for SequentialIds {
    fn new_id(&self) -> Uuid {
        Self::id(self.made.fetch_add(1, Ordering::Relaxed) + 1)
    }
}
