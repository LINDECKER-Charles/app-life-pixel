use super::change::Change;
use crate::model::Animation;

/// What undoes an applied operation: the cels, palette, layers, frames, tags or title it
/// replaced. Applying it returns the inverse that redoes the operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inverse(Change);

impl Inverse {
    pub(crate) fn new(change: Change) -> Self {
        Self(change)
    }

    /// Puts the replaced values back into `animation`, the animation the operation was applied
    /// to, and returns the inverse of that.
    #[must_use]
    pub fn apply(self, animation: &mut Animation) -> Self {
        Self(self.0.swap_into(animation))
    }

    /// Whether the operation changed nothing, so that there is nothing to undo.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Roughly the memory it holds, in bytes: what the history's byte limit counts.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.0.byte_size()
    }
}
