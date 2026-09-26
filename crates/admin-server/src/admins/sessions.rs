//! Admin sessions: a random token in the cookie, only its SHA-256 in the database, and short
//! lives — 30 minutes idle, 8 hours in all.

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

/// A session ends after this long without a request.
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::minutes(30);
/// A session ends this long after it began, however busy.
pub const SESSION_LIFETIME: Duration = Duration::hours(8);
/// A session's `last_seen_at` moves at most this often, to spare the database a write per
/// request.
pub const SESSION_TOUCH_INTERVAL: Duration = Duration::minutes(1);
/// The random bytes of a token.
const TOKEN_BYTES: usize = 32;

/// A session's secret, as the cookie carries it: 32 random bytes in base64url. Its `Debug` hides
/// it.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    /// A new random token.
    #[must_use]
    pub fn generate() -> Self {
        let bytes: [u8; TOKEN_BYTES] = rand::random();
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    /// The token a cookie carries, when it has the shape of one.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let bytes = URL_SAFE_NO_PAD.decode(value).ok()?;
        (bytes.len() == TOKEN_BYTES).then(|| Self(value.to_owned()))
    }

    /// The token, for the cookie.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// The hash the database keeps.
    #[must_use]
    pub fn hash(&self) -> TokenHash {
        TokenHash(Sha256::digest(self.0.as_bytes()).into())
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionToken(<redacted>)")
    }
}

/// The SHA-256 of a session's token: the key of `admin_sessions`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenHash([u8; 32]);

impl TokenHash {
    /// The hash's bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// When a session began, was last seen, and ends at the latest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionTimes {
    /// When it began.
    pub created_at: OffsetDateTime,
    /// The last request it made, to the minute.
    pub last_seen_at: OffsetDateTime,
    /// 8 hours after it began.
    pub expires_at: OffsetDateTime,
}

impl SessionTimes {
    /// The times of a session beginning `now`.
    #[must_use]
    pub fn starting(now: OffsetDateTime) -> Self {
        Self {
            created_at: now,
            last_seen_at: now,
            expires_at: now + SESSION_LIFETIME,
        }
    }

    /// Whether the session is still live `now`: neither idle 30 minutes nor 8 hours old.
    #[must_use]
    pub fn is_live(&self, now: OffsetDateTime) -> bool {
        now < self.expires_at && now < self.last_seen_at + SESSION_IDLE_TIMEOUT
    }

    /// Whether a request `now` should move `last_seen_at`.
    #[must_use]
    pub fn is_due_for_touch(&self, now: OffsetDateTime) -> bool {
        now - self.last_seen_at >= SESSION_TOUCH_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap()
    }

    #[test]
    fn a_session_ends_after_thirty_idle_minutes() {
        let times = SessionTimes::starting(start());
        assert!(times.is_live(start() + Duration::minutes(29)));
        assert!(!times.is_live(start() + Duration::minutes(30)));
    }

    #[test]
    fn a_busy_session_still_ends_eight_hours_after_it_began() {
        let mut times = SessionTimes::starting(start());
        let late = start() + Duration::hours(8) - Duration::seconds(1);
        times.last_seen_at = late - Duration::minutes(1);
        assert!(times.is_live(late));
        assert!(!times.is_live(start() + Duration::hours(8)));
    }

    #[test]
    fn last_seen_moves_once_a_minute_at_most() {
        let times = SessionTimes::starting(start());
        assert!(!times.is_due_for_touch(start() + Duration::seconds(59)));
        assert!(times.is_due_for_touch(start() + Duration::seconds(60)));
    }

    #[test]
    fn a_token_is_32_random_bytes_and_only_its_hash_is_kept() {
        let token = SessionToken::generate();
        assert_eq!(token.expose().len(), 43);
        assert_eq!(SessionToken::parse(token.expose()), Some(token.clone()));
        assert_eq!(SessionToken::parse("short"), None);
        assert_ne!(token.hash(), SessionToken::generate().hash());
        assert!(!format!("{token:?}").contains(token.expose()));
    }
}
