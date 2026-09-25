//! What a tool may do, as a personal access token grants it.

/// The permission a tool needs: a token's scopes grant them one by one (A3); the local user has
/// every one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Reading animations, their previews and their snippets.
    Read,
    /// Creating and changing animations.
    Write,
    /// Exporting animations as files.
    Export,
}

impl Scope {
    /// The scope's name, as a token lists it: `read`, `write` or `export`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Export => "export",
        }
    }
}
