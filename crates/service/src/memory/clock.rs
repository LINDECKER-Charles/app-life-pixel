//! A clock that tells the time it is set to.

use std::sync::{Mutex, PoisonError};

use time::{Duration, OffsetDateTime};

use crate::ports::Clock;

/// A clock standing still until it is set or advanced.
#[derive(Debug)]
pub struct FixedClock {
    now: Mutex<OffsetDateTime>,
}

impl FixedClock {
    /// A clock showing `now`.
    #[must_use]
    pub fn new(now: OffsetDateTime) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    /// Sets the clock to `now`.
    pub fn set(&self, now: OffsetDateTime) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) = now;
    }

    /// Moves the clock `duration` forward.
    pub fn advance(&self, duration: Duration) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) += duration;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
