//! How agents address an animation's parts: frames by position, layers by id, the top layer by
//! default.

use life_pixel_core::edit::{self, EditError, Operation};
use life_pixel_core::{Animation, Frame, FrameId, Layer, LayerId};

/// A frame and a layer to draw on, as an agent names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    /// The frame's position, from 0.
    pub(super) frame: u16,
    /// The layer's id; `None` for the top layer.
    pub(super) layer: Option<u32>,
}

impl Target {
    /// The ids of the layer and frame, once the frame's position is found. Whether the layer
    /// exists is checked by the operations that use it.
    pub(super) fn resolve(self, animation: &Animation) -> Result<(LayerId, FrameId), EditError> {
        let frame = frame_at(animation, self.frame)?;
        let layer = self
            .layer
            .map_or_else(|| top_layer(animation), LayerId::new);
        Ok((layer, frame))
    }
}

/// The id of the frame at `position`.
pub(super) fn frame_at(animation: &Animation, position: u16) -> Result<FrameId, EditError> {
    let frame = animation.frames().get(usize::from(position));
    frame.map(Frame::id).ok_or(EditError::FrameNotFound)
}

/// Applies `operations` in order with `core`; the first refused stops the others.
pub(super) fn apply_all(
    animation: &mut Animation,
    operations: &[Operation],
) -> Result<(), EditError> {
    for operation in operations {
        let _undo = edit::apply(animation, operation)?;
    }
    Ok(())
}

/// The id of the top layer; an animation always has one.
fn top_layer(animation: &Animation) -> LayerId {
    let top = animation.layers().last().map(Layer::id);
    top.unwrap_or(LayerId::new(0))
}
