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
    /// A package depends on one its side of the licence boundary, or its layer, forbids.
    ForbiddenDependency {
        /// The package.
        package: String,
        /// The dependency it must not have.
        dependency: String,
    },
    /// A workspace member has no place in the dependency direction.
    UndeclaredMember {
        /// The member.
        package: String,
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
            Self::UndeclaredMember { package } => {
                write!(
                    formatter,
                    "{package} is missing from the layers of xtask/src/boundaries/architecture.rs"
                )
            }
        }
    }
}
