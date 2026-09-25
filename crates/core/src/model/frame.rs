use serde::{Deserialize, Serialize};

use crate::error::DocumentError;
use crate::limits::{MAX_FRAME_DURATION_MS, MIN_FRAME_DURATION_MS};

/// A frame's id, local to its animation and taken from its `nextId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameId(u32);

impl FrameId {
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

/// A frame of an animation: how long it shows. Its pixels are the cels of its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Frame {
    id: FrameId,
    duration_ms: u16,
}

impl Frame {
    /// The frame `id`, shown for `duration_ms` milliseconds.
    ///
    /// # Errors
    ///
    /// [`DocumentError::FrameDuration`] when the duration is out of [`MIN_FRAME_DURATION_MS`] to
    /// [`MAX_FRAME_DURATION_MS`].
    pub fn new(id: FrameId, duration_ms: u16) -> Result<Self, DocumentError> {
        let is_valid = (MIN_FRAME_DURATION_MS..=MAX_FRAME_DURATION_MS).contains(&duration_ms);
        is_valid
            .then_some(Self { id, duration_ms })
            .ok_or(DocumentError::FrameDuration)
    }

    /// The frame's id.
    #[must_use]
    pub fn id(&self) -> FrameId {
        self.id
    }

    /// How long the frame shows, in milliseconds.
    #[must_use]
    pub fn duration_ms(&self) -> u16 {
        self.duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_lasts_10_ms_at_least() {
        let id = FrameId::new(1);
        assert!(Frame::new(id, MIN_FRAME_DURATION_MS).is_ok());
        assert!(Frame::new(id, MAX_FRAME_DURATION_MS).is_ok());
        let too_short = Frame::new(id, MIN_FRAME_DURATION_MS - 1);
        assert_eq!(too_short, Err(DocumentError::FrameDuration));
    }
}
