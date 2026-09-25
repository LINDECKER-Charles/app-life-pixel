use std::fmt;

/// A broken boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    /// A package declares another licence than its side's.
    WrongLicense {
        /// The package.
        package: String,
        /// The licence of its side.
        expected: &'static str,
        /// The licence it declares, if any.
        found: Option<String>,
    },
    /// An MIT crate depends on a package it must not.
    ForbiddenDependency {
        /// The MIT crate.
        package: String,
        /// The dependency it must not have.
        dependency: String,
    },
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLicense {
                package,
                expected,
                found,
            } => {
                let found = found.as_deref().unwrap_or("no licence");
                write!(formatter, "{package} must declare {expected}, not {found}")
            }
            Self::ForbiddenDependency {
                package,
                dependency,
            } => {
                write!(formatter, "{package} must not depend on {dependency}")
            }
        }
    }
}
