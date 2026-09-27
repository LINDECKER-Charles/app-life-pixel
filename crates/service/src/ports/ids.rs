//! New ids, injected so that tests make them predictable.

use uuid::Uuid;

/// Makes new ids.
pub trait IdGenerator: Send + Sync {
    /// A new, unique id: a UUIDv7.
    fn new_id(&self) -> Uuid;
}

/// UUIDv7s from the system's clock and random generator.
#[derive(Clone, Copy, Debug, Default)]
pub struct UuidV7Ids;

impl IdGenerator for UuidV7Ids {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}
