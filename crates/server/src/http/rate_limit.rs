//! Rate limits: in-process keyed limiters, one server instance per environment. A route checks
//! its policy; the API routes without one take `api` through [`limit_api`].

mod policies;

use std::collections::HashMap;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use governor::clock::Clock;
use governor::{DefaultKeyedRateLimiter, Quota};
use life_pixel_service::AccountId;

use self::policies::{KeyKind, LIMITS, Limit};
use super::client_address::{ClientAddress, peer_address};
use super::problem::{Problem, codes};
use crate::state::AppState;

pub use policies::{Policy, RateKey};

/// How often the limiters forget the keys that are back to a full allowance.
pub const RATE_LIMIT_UPKEEP_PERIOD: Duration = Duration::from_secs(60);

/// Every limiter of the policy table.
pub struct RateLimits {
    limiters: HashMap<(Policy, KeyKind), DefaultKeyedRateLimiter<RateKey>>,
}

impl RateLimits {
    /// Fresh limiters, one per line of the policy table.
    #[must_use]
    pub fn new() -> Self {
        let limiters = LIMITS
            .iter()
            .map(|limit| {
                (
                    (limit.policy, limit.key),
                    DefaultKeyedRateLimiter::keyed(quota(limit)),
                )
            })
            .collect();
        Self { limiters }
    }

    /// Counts one request of `key` under `policy`.
    ///
    /// # Errors
    ///
    /// `rate_limit.exceeded`, with the seconds to wait, once the limit is reached.
    pub fn check(&self, policy: Policy, key: &RateKey) -> Result<(), Problem> {
        let Some(limiter) = self.limiters.get(&(policy, key.kind())) else {
            tracing::error!(?policy, kind = ?key.kind(), "no rate limit for this kind of key");
            return Ok(());
        };
        limiter.check_key(key).map_err(|not_until| {
            let wait = not_until.wait_time_from(limiter.clock().now());
            exceeded(wait)
        })
    }

    /// Forgets the keys whose allowance is full again, so that memory follows the traffic.
    pub fn retain_recent(&self) {
        for limiter in self.limiters.values() {
            limiter.retain_recent();
            limiter.shrink_to_fit();
        }
    }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware of the API routes without a policy of their own: `api`, by the account a layer
/// before it put in the request's extensions, else by the client's address.
///
/// # Errors
///
/// `rate_limit.exceeded` once the limit is reached.
pub async fn limit_api(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Problem> {
    let key = match request.extensions().get::<AccountId>() {
        Some(account) => RateKey::Account(*account),
        None => {
            let peer = peer_address(request.extensions());
            let proxies = &state.config.trusted_proxies;
            RateKey::Address(ClientAddress::resolve(peer, request.headers(), proxies).0)
        }
    };
    state.rate_limits.check(Policy::Api, &key)?;
    Ok(next.run(request).await)
}

/// `count` requests at once, one more every `window / count`.
fn quota(limit: &Limit) -> Quota {
    let replenish_one_per = limit.window / limit.count.get();
    Quota::with_period(replenish_one_per).map_or(Quota::per_second(limit.count), |quota| {
        quota.allow_burst(limit.count)
    })
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
    fn email_keys_ignore_case_and_count_apart_from_addresses() {
        let limits = RateLimits::new();
        for _ in 0..5 {
            limits
                .check(Policy::SignIn, &RateKey::email("Ada@Example.org"))
                .unwrap();
        }
        let refused = limits.check(Policy::SignIn, &RateKey::email(" ada@example.org"));
        let problem = refused.unwrap_err();
        assert_eq!(problem.code, codes::RATE_LIMIT_EXCEEDED);
        assert_eq!(problem.retry_after, Some(12));
        let address = RateKey::Address([192, 0, 2, 1].into());
        assert!(limits.check(Policy::SignIn, &address).is_ok());
    }

    #[test]
    fn a_daily_policy_refills_one_request_per_share_of_the_day() {
        let limits = RateLimits::new();
        let account = RateKey::Account(AccountId::from_uuid(uuid::Uuid::nil()));
        for _ in 0..10 {
            limits.check(Policy::SupportCreate, &account).unwrap();
        }
        let problem = limits.check(Policy::SupportCreate, &account).unwrap_err();
        assert_eq!(problem.retry_after, Some(8_640));
    }
}
