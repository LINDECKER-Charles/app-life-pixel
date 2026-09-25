use serde::{Deserialize, Serialize};

use super::Name;

/// A layer's id, local to its animation and taken from its `nextId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerId(u32);

impl LayerId {
    /// The id `value`.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// The id's number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A layer of an animation. Layers stack from bottom to top.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Layer {
    id: LayerId,
    name: Name,
    visible: bool,
}

impl Layer {
    /// A visible layer.
    #[must_use]
    pub fn new(id: LayerId, name: Name) -> Self {
        Self {
            id,
            name,
            visible: true,
        }
    }

    /// The same layer, shown or hidden.
    #[must_use]
    pub fn with_visibility(self, visible: bool) -> Self {
        Self { visible, ..self }
    }

    /// The layer's id.
    #[must_use]
    pub fn id(&self) -> LayerId {
        self.id
    }

    /// The layer's name.
    #[must_use]
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Whether the layer is shown: a hidden layer is left out of compositing.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}
