use super::Name;

/// A project: a named group of animations. Its id is a UUID made by the service.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Project {
    name: Name,
}

impl Project {
    /// The project `name`.
    #[must_use]
    pub fn new(name: Name) -> Self {
        Self { name }
    }

    /// The project's name.
    #[must_use]
    pub fn name(&self) -> &Name {
        &self.name
    }
}
