use crate::error::DocumentError;
use crate::limits::DEFAULT_FRAME_DURATION_MS;
use crate::model::{Frame, FrameId, Layer, LayerId, Name, Palette};

use super::{Animation, AnimationParts};

/// The id of a new animation's layer; its frame takes the next one.
const FIRST_ID: u32 = 1;

/// What a new animation is made of: one layer, one blank frame, no tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAnimation {
    /// The title.
    pub title: Name,
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// The name of its layer, translated by the caller.
    pub layer_name: Name,
    /// The duration of its frame; [`DEFAULT_FRAME_DURATION_MS`] when `None`.
    pub frame_duration_ms: Option<u16>,
    /// Its palette; [`DEFAULT_PALETTE`](crate::DEFAULT_PALETTE) when `None`.
    pub palette: Option<Palette>,
}

impl Animation {
    /// A new animation: one visible layer with id 1, one blank frame with id 2, no tag.
    ///
    /// # Errors
    ///
    /// [`DocumentError::CanvasSize`] or [`DocumentError::FrameDuration`] when the size or the
    /// duration is out of its limits.
    pub fn new(spec: NewAnimation) -> Result<Self, DocumentError> {
        let layer_id = LayerId::new(FIRST_ID);
        let frame_id = FrameId::new(FIRST_ID + 1);
        let duration_ms = spec.frame_duration_ms.unwrap_or(DEFAULT_FRAME_DURATION_MS);
        Self::from_parts(AnimationParts {
            title: spec.title,
            width: spec.width,
            height: spec.height,
            palette: spec.palette.unwrap_or_default(),
            layers: vec![Layer::new(layer_id, spec.layer_name)],
            frames: vec![Frame::new(frame_id, duration_ms)?],
            tags: Vec::new(),
            next_id: FIRST_ID + 2,
        })
    }
}
