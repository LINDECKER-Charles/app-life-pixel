//! What an operation refers to, checked before anything else: layers, frames, tags, palette
//! entries and positions.

use super::EditError;
use crate::error::DocumentError;
use crate::model::{Animation, FrameId, LayerId};

/// The palette entry an operation may never change: transparent, always.
const TRANSPARENT_INDEX: u8 = 0;

/// The position of the layer `id`, from the bottom.
pub(crate) fn layer_position(animation: &Animation, id: LayerId) -> Result<usize, EditError> {
    let mut layers = animation.layers().iter();
    layers
        .position(|layer| layer.id() == id)
        .ok_or(EditError::LayerNotFound)
}

/// The position of the frame `id`, in play order.
pub(crate) fn frame_position(animation: &Animation, id: FrameId) -> Result<usize, EditError> {
    let mut frames = animation.frames().iter();
    frames
        .position(|frame| frame.id() == id)
        .ok_or(EditError::FrameNotFound)
}

/// The position of the tag `name` in the animation's list.
pub(crate) fn tag_position(animation: &Animation, name: &str) -> Result<usize, EditError> {
    let mut tags = animation.tags().iter();
    tags.position(|tag| tag.name().as_str() == name)
        .ok_or_else(|| EditError::TagNotFound {
            name: name.to_owned(),
        })
}

/// `index`, when the palette has it.
pub(crate) fn palette_index(animation: &Animation, index: u32) -> Result<u8, EditError> {
    u8::try_from(index)
        .ok()
        .filter(|&index| usize::from(index) < animation.palette().len())
        .ok_or(EditError::Document(DocumentError::Palette))
}

/// `index`, when the palette has it and it is not the transparent entry 0.
pub(crate) fn editable_palette_index(animation: &Animation, index: u32) -> Result<u8, EditError> {
    let index = palette_index(animation, index)?;
    (index != TRANSPARENT_INDEX)
        .then_some(index)
        .ok_or(EditError::Document(DocumentError::Palette))
}

/// `position` in a list where `max` is the highest position allowed.
pub(crate) fn position(position: u32, max: usize) -> Result<usize, EditError> {
    usize::try_from(position)
        .ok()
        .filter(|&position| position <= max)
        .ok_or(EditError::PositionOutOfRange { max })
}

/// `duration_ms` in the width a frame holds it; its range is the frame's own rule.
pub(crate) fn frame_duration(duration_ms: u32) -> Result<u16, EditError> {
    u16::try_from(duration_ms).map_err(|_| EditError::Document(DocumentError::FrameDuration))
}
