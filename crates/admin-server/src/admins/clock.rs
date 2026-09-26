//! The time sign-in and sessions read: the system's, or a test's.

use time::OffsetDateTime;

/// Tells the time.
pub trait Clock: Send + Sync {
    /// Now.
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
