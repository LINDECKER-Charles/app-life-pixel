//! The sign-in rate limit: 5 attempts a minute per client address and per email, in process —
//! one admin server per environment.

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use governor::clock::Clock as _;
use governor::{DefaultKeyedRateLimiter, Quota};

use crate::http::problem::{Problem, codes};

/// Sign-in attempts allowed a minute, per address and per email.
const ATTEMPTS_PER_MINUTE: NonZeroU32 = match NonZeroU32::new(5) {
    Some(attempts) => attempts,
    None => NonZeroU32::MIN,
};
/// How often the limiters forget the keys that are back to a full allowance.
pub const RATE_LIMIT_UPKEEP_PERIOD: Duration = Duration::from_secs(60);

/// The two limiters of sign-in.
pub struct SignInLimits {
    by_address: DefaultKeyedRateLimiter<IpAddr>,
    by_email: DefaultKeyedRateLimiter<String>,
}

impl SignInLimits {
    /// Fresh limiters.
    #[must_use]
    pub fn new() -> Self {
        let quota = Quota::per_minute(ATTEMPTS_PER_MINUTE);
        Self {
            by_address: DefaultKeyedRateLimiter::keyed(quota),
            by_email: DefaultKeyedRateLimiter::keyed(quota),
        }
    }

    /// Counts one attempt from `address` for `email`, whatever its case.
    ///
    /// # Errors
    ///
    /// `rate_limit.exceeded`, with the seconds to wait, once either limit is reached.
    pub fn check(&self, address: IpAddr, email: &str) -> Result<(), Problem> {
        let email = email.trim().to_lowercase();
        self.by_address.check_key(&address).map_err(|not_until| {
            exceeded(not_until.wait_time_from(self.by_address.clock().now()))
        })?;
        self.by_email
            .check_key(&email)
            .map_err(|not_until| exceeded(not_until.wait_time_from(self.by_email.clock().now())))
    }

    /// Forgets the keys whose allowance is full again.
    pub fn retain_recent(&self) {
        self.by_address.retain_recent();
        self.by_email.retain_recent();
    }
}

impl Default for SignInLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// `rate_limit.exceeded`, the wait rounded up to whole seconds, at least one.
fn exceeded(wait: Duration) -> Problem {
    let whole_seconds = wait.as_secs() + u64::from(wait.subsec_nanos() > 0);
    let seconds = u32::try_from(whole_seconds.max(1)).unwrap_or(u32::MAX);
    Problem::new(codes::RATE_LIMIT_EXCEEDED)
        .with_param("retryAfterSeconds", seconds)
        .with_retry_after(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_attempts_a_minute_per_email_whatever_the_address() {
        let limits = SignInLimits::new();
        for last in 0..5 {
            let address = IpAddr::from([192, 0, 2, last]);
            limits.check(address, "Ada@Example.org").unwrap();
        }
        let refused = limits.check(IpAddr::from([192, 0, 2, 9]), " ada@example.org");
        let problem = refused.unwrap_err();
        assert_eq!(problem.code, codes::RATE_LIMIT_EXCEEDED);
        assert_eq!(problem.retry_after, Some(12));
    }

    #[test]
    fn five_attempts_a_minute_per_address_whatever_the_email() {
        let limits = SignInLimits::new();
        let address = IpAddr::from([192, 0, 2, 1]);
        for index in 0..5 {
            limits
                .check(address, &format!("admin{index}@example.org"))
                .unwrap();
        }
        assert!(limits.check(address, "other@example.org").is_err());
    }
}
