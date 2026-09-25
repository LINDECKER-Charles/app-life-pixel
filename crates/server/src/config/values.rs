//! The value types of the configuration: environment, origins, secrets, log filter.

use std::fmt;

use tracing_subscriber::EnvFilter;

use super::env::FromVariable;

/// Where the server runs: in logs and traces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment {
    /// A developer's machine.
    Local,
    /// The staging host.
    Staging,
    /// The production host, or a self-hosted server.
    Production,
}

impl Environment {
    /// The name, as `LP_ENVIRONMENT` spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

impl FromVariable for Environment {
    const EXPECTED: &'static str = "local, staging or production";

    fn from_variable(value: &str) -> Option<Self> {
        [Self::Local, Self::Staging, Self::Production]
            .into_iter()
            .find(|environment| environment.as_str() == value)
    }
}

const HTTP_SCHEME: &str = "http://";
const HTTPS_SCHEME: &str = "https://";

/// An origin, `scheme://host[:port]` over http or https, without a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin(String);

impl Origin {
    /// The origin, as a browser sends it in `Origin`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the origin is served over TLS.
    #[must_use]
    pub fn is_https(&self) -> bool {
        self.0.starts_with(HTTPS_SCHEME)
    }
}

impl FromVariable for Origin {
    const EXPECTED: &'static str = "an origin such as https://lifepixel.tech, without a path";

    fn from_variable(value: &str) -> Option<Self> {
        let origin = value.strip_suffix('/').unwrap_or(value);
        let host = origin
            .strip_prefix(HTTPS_SCHEME)
            .or_else(|| origin.strip_prefix(HTTP_SCHEME))?;
        let is_host = !host.is_empty() && !host.contains(['/', '?', '#', '@', ' ', ',']);
        is_host.then(|| Self(origin.to_owned()))
    }
}

impl FromVariable for Vec<Origin> {
    const EXPECTED: &'static str = "origins separated by commas, such as http://localhost:4260";

    fn from_variable(value: &str) -> Option<Self> {
        value
            .split(',')
            .map(|origin| Origin::from_variable(origin.trim()))
            .collect()
    }
}

/// A value that must never reach a log: its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    /// The value, for the one place that needs it.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString(<redacted>)")
    }
}

impl FromVariable for SecretString {
    const EXPECTED: &'static str = "a text";

    fn from_variable(value: &str) -> Option<Self> {
        Some(Self(value.to_owned()))
    }
}

/// The length of an HMAC key, in bytes.
pub const HMAC_KEY_BYTES: usize = 32;

/// A 256-bit HMAC key, written as 64 hexadecimal characters. Its `Debug` hides it.
#[derive(Clone, PartialEq, Eq)]
pub struct HmacKey([u8; HMAC_KEY_BYTES]);

impl HmacKey {
    /// The key's bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; HMAC_KEY_BYTES] {
        &self.0
    }
}

impl fmt::Debug for HmacKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HmacKey(<redacted>)")
    }
}

impl FromVariable for HmacKey {
    const EXPECTED: &'static str = "64 hexadecimal characters";

    fn from_variable(value: &str) -> Option<Self> {
        let digits = value.as_bytes();
        if digits.len() != HMAC_KEY_BYTES * 2 {
            return None;
        }
        let mut key = [0; HMAC_KEY_BYTES];
        let (pairs, _) = digits.as_chunks::<2>();
        for (byte, [high, low]) in key.iter_mut().zip(pairs) {
            *byte = (hex_value(*high)? << 4) | hex_value(*low)?;
        }
        Some(Self(key))
    }
}

/// The value of a hexadecimal digit, either case.
fn hex_value(digit: u8) -> Option<u8> {
    let value = char::from(digit).to_digit(16)?;
    u8::try_from(value).ok()
}

/// The log filter when `RUST_LOG` is unset.
const DEFAULT_LOG_FILTER: &str = "info";

/// `RUST_LOG`, checked to parse as a `tracing` filter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFilter(String);

impl LogFilter {
    /// The filter's directives.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for LogFilter {
    fn default() -> Self {
        Self(DEFAULT_LOG_FILTER.to_owned())
    }
}

impl FromVariable for LogFilter {
    const EXPECTED: &'static str = "tracing directives such as info or life_pixel_server=debug";

    fn from_variable(value: &str) -> Option<Self> {
        EnvFilter::try_new(value)
            .ok()
            .map(|_| Self(value.to_owned()))
    }
}
