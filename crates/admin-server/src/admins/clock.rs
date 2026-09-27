//! The time sign-in and sessions read: the system's, or a test's.

use time::OffsetDateTime;

/// Tells the time.
pub trait Clock: Send + Sync {
    /// Now.
    fn now(&self) -> OffsetDateTime;
}

/// The system's clock, to the microsecond: the precision Postgres's `timestamptz` keeps, so that
/// a time handed back equals the one read again later.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        let now = OffsetDateTime::now_utc();
        now.replace_microsecond(now.microsecond()).unwrap_or(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NANOSECONDS_PER_MICROSECOND: u32 = 1_000;

    #[test]
    fn the_system_clock_keeps_whole_microseconds() {
        assert_eq!(
            SystemClock.now().nanosecond() % NANOSECONDS_PER_MICROSECOND,
            0
        );
    }
}
