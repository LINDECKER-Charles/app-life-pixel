//! Reading variables: required, optional or defaulted, each parsed into its type, and the error
//! that names the variable without ever quoting its value.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::PathBuf;

use thiserror::Error;

/// A variable the server cannot start without. The message names the variable and what it
/// expects, never its value: some values are secrets.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// The variable is unset or blank.
    #[error("{variable} is missing")]
    Missing {
        /// The variable's name.
        variable: &'static str,
    },
    /// The variable's value does not parse.
    #[error("{variable} is invalid: expected {expected}")]
    Invalid {
        /// The variable's name.
        variable: &'static str,
        /// What a valid value looks like.
        expected: &'static str,
    },
}

impl ConfigError {
    /// The variable at fault.
    #[must_use]
    pub fn variable(&self) -> &'static str {
        match self {
            Self::Missing { variable } | Self::Invalid { variable, .. } => variable,
        }
    }
}

/// A type a variable's value parses into.
pub trait FromVariable: Sized {
    /// What a valid value looks like, for the error message.
    const EXPECTED: &'static str;

    /// The value parsed, or `None` when it is invalid. `value` is trimmed and not empty.
    fn from_variable(value: &str) -> Option<Self>;
}

/// The variables, through a lookup: the process environment, or a map in tests.
pub struct Env<'a> {
    lookup: &'a dyn Fn(&str) -> Option<String>,
}

impl<'a> Env<'a> {
    /// Reads variables through `lookup`.
    pub fn new(lookup: &'a dyn Fn(&str) -> Option<String>) -> Self {
        Self { lookup }
    }

    /// The trimmed value of `variable`, or `None` when it is unset or blank.
    pub fn optional(&self, variable: &'static str) -> Option<String> {
        (self.lookup)(variable)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    }

    /// The value of `variable` parsed into `T`; unset or blank is an error.
    pub fn parse<T: FromVariable>(&self, variable: &'static str) -> Result<T, ConfigError> {
        let value = self
            .optional(variable)
            .ok_or(ConfigError::Missing { variable })?;
        parse_value(variable, &value)
    }

    /// The value of `variable` parsed into `T`, or `T`'s default when unset or blank.
    pub fn parse_or_default<T>(&self, variable: &'static str) -> Result<T, ConfigError>
    where
        T: FromVariable + Default,
    {
        self.optional(variable)
            .map_or_else(|| Ok(T::default()), |value| parse_value(variable, &value))
    }
}

fn parse_value<T: FromVariable>(variable: &'static str, value: &str) -> Result<T, ConfigError> {
    T::from_variable(value).ok_or(ConfigError::Invalid {
        variable,
        expected: T::EXPECTED,
    })
}

impl FromVariable for String {
    const EXPECTED: &'static str = "a text";

    fn from_variable(value: &str) -> Option<Self> {
        Some(value.to_owned())
    }
}

impl FromVariable for PathBuf {
    const EXPECTED: &'static str = "a path";

    fn from_variable(value: &str) -> Option<Self> {
        Some(Self::from(value))
    }
}

impl FromVariable for SocketAddr {
    const EXPECTED: &'static str = "an address and a port, such as 127.0.0.1:8460";

    fn from_variable(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromVariable for NonZeroU32 {
    const EXPECTED: &'static str = "a whole number above zero";

    fn from_variable(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromVariable for u32 {
    const EXPECTED: &'static str = "a whole number below 2^32";

    fn from_variable(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromVariable for u64 {
    const EXPECTED: &'static str = "a whole number below 2^64";

    fn from_variable(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromVariable for bool {
    const EXPECTED: &'static str = "true or false";

    fn from_variable(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}
