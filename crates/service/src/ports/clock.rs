//! The time of the use cases, injected so that tests fix it.

use time::OffsetDateTime;

/// The current time.
pub trait Clock: Send + Sync {
    /// Now, in UTC.
    fn now(&self) -> OffsetDateTime;
}

/// The system's clock, to the microsecond: the precision every store keeps (Postgres's
/// `timestamptz`), so that what a use case returns equals what a later read finds.
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
