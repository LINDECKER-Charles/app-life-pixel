//! TOTP, RFC 6238 as authenticators compute it: HMAC-SHA-1, 6 digits, 30-second steps. A code is
//! accepted for the current step or one step either side, and only for a step after the last
//! one used, so that a code works once.

use std::fmt;

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;

/// The length of a TOTP secret, in bytes: 160 bits, as RFC 4226 recommends for HMAC-SHA-1.
pub const TOTP_SECRET_BYTES: usize = 20;
/// The digits of a code.
pub const TOTP_DIGITS: usize = 6;
/// The length of a step, in seconds.
pub const TOTP_STEP_SECONDS: i64 = 30;
/// The steps accepted either side of the current one, for a clock that drifts.
pub const TOTP_DRIFT_STEPS: i64 = 1;
/// 10 to the power of the digits.
const CODE_MODULUS: u32 = 1_000_000;

/// A TOTP secret. Its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct TotpSecret([u8; TOTP_SECRET_BYTES]);

impl TotpSecret {
    /// A new random secret.
    #[must_use]
    pub fn generate() -> Self {
        Self(rand::random())
    }

    /// The secret of `bytes`, or `None` when they are not 20 bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes.try_into().ok().map(Self)
    }

    /// The secret's bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; TOTP_SECRET_BYTES] {
        &self.0
    }

    /// The secret in base32 without padding, as `otpauth://` URIs carry it.
    #[must_use]
    pub fn to_base32(&self) -> String {
        base32(&self.0)
    }

    /// The code of `step`, as six digits.
    #[must_use]
    pub fn code_at(&self, step: i64) -> String {
        let counter = u64::try_from(step).unwrap_or_default().to_be_bytes();
        let mut mac = <Hmac<Sha1> as KeyInit>::new_from_slice(&self.0)
            .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
        mac.update(&counter);
        let digest = mac.finalize().into_bytes();
        let offset = usize::from(digest[digest.len() - 1] & 0x0f);
        let word = [
            digest[offset] & 0x7f,
            digest[offset + 1],
            digest[offset + 2],
            digest[offset + 3],
        ];
        let code = u32::from_be_bytes(word) % CODE_MODULUS;
        format!("{code:0width$}", width = TOTP_DIGITS)
    }

    /// The step `code` is valid for at `now`, when it is one of the steps around now's that come
    /// after `last_step`: the latest such step matching.
    #[must_use]
    pub fn verify(&self, code: &str, (now, last_step): (OffsetDateTime, i64)) -> Option<i64> {
        if code.len() != TOTP_DIGITS || !code.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let current = step_at(now);
        let steps = (current - TOTP_DRIFT_STEPS..=current + TOTP_DRIFT_STEPS).rev();
        let mut found = None;
        for step in steps.filter(|step| *step > last_step) {
            let is_match: bool = self.code_at(step).as_bytes().ct_eq(code.as_bytes()).into();
            if is_match && found.is_none() {
                found = Some(step);
            }
        }
        found
    }
}

impl fmt::Debug for TotpSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TotpSecret(<redacted>)")
    }
}

/// The step `now` falls in: whole 30-second periods since the Unix epoch.
#[must_use]
pub fn step_at(now: OffsetDateTime) -> i64 {
    now.unix_timestamp().div_euclid(TOTP_STEP_SECONDS)
}

/// RFC 4648 base32, without padding.
fn base32(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    const BITS_PER_CHARACTER: u32 = 5;
    let mut text = String::new();
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= BITS_PER_CHARACTER {
            bits -= BITS_PER_CHARACTER;
            text.push(char::from(ALPHABET[((buffer >> bits) & 0x1f) as usize]));
        }
    }
    if bits > 0 {
        let index = (buffer << (BITS_PER_CHARACTER - bits)) & 0x1f;
        text.push(char::from(ALPHABET[index as usize]));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6238's SHA-1 secret, "12345678901234567890".
    fn rfc_secret() -> TotpSecret {
        TotpSecret::from_bytes(b"12345678901234567890").unwrap()
    }

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(seconds).unwrap()
    }

    #[test]
    fn codes_are_those_of_rfc_6238() {
        let secret = rfc_secret();
        for (seconds, code) in [
            (59, "287082"),
            (1_111_111_109, "081804"),
            (1_234_567_890, "005924"),
            (2_000_000_000, "279037"),
        ] {
            assert_eq!(secret.code_at(step_at(at(seconds))), code, "{seconds}");
        }
    }

    #[test]
    fn a_valid_code_gives_its_step() {
        let now = at(1_234_567_890);
        let code = rfc_secret().code_at(step_at(now));
        assert_eq!(rfc_secret().verify(&code, (now, 0)), Some(step_at(now)));
    }

    #[test]
    fn a_code_one_step_away_is_accepted_and_two_steps_away_refused() {
        let now = at(1_234_567_890);
        let secret = rfc_secret();
        let current = step_at(now);
        for step in [current - 1, current + 1] {
            assert_eq!(secret.verify(&secret.code_at(step), (now, 0)), Some(step));
        }
        for step in [current - 2, current + 2] {
            assert_eq!(secret.verify(&secret.code_at(step), (now, 0)), None);
        }
    }

    #[test]
    fn a_replayed_code_is_refused() {
        let now = at(1_234_567_890);
        let secret = rfc_secret();
        let step = step_at(now);
        let code = secret.code_at(step);
        assert_eq!(secret.verify(&code, (now, step)), None);
        let earlier = secret.code_at(step - 1);
        assert_eq!(secret.verify(&earlier, (now, step)), None);
    }

    #[test]
    fn a_wrong_or_malformed_code_is_refused() {
        let now = at(1_234_567_890);
        let secret = rfc_secret();
        let code = secret.code_at(step_at(now));
        let wrong = format!("{:06}", (code.parse::<u32>().unwrap() + 1) % CODE_MODULUS);
        for sent in [wrong.as_str(), "", "12345", "1234567", "12a456", " 00592"] {
            assert_eq!(secret.verify(sent, (now, 0)), None, "{sent:?}");
        }
    }

    #[test]
    fn secrets_are_twenty_random_bytes_written_in_base32() {
        assert_eq!(rfc_secret().to_base32(), "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        let secret = TotpSecret::generate();
        assert_eq!(secret.to_base32().len(), 32);
        assert_ne!(secret, TotpSecret::generate());
        assert!(!format!("{secret:?}").contains(&secret.to_base32()));
    }
}
