//! The time of the use cases, injected so that tests fix it.

use time::OffsetDateTime;

/// The current time.
pub trait Clock: Send + Sync {
    /// Now, in UTC.
    fn now(&self) -> OffsetDateTime;
}

/// The system's clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
