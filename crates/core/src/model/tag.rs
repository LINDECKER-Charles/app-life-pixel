use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::TagName;

/// How a tag plays when it reaches its last frame. Written `"loop"` or `"once"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    /// Goes back to the tag's first frame.
    Loop,
    /// Stays on the tag's last frame and stops.
    Once,
}

/// A named range of frames, by position from 0, both ends included. Its animation checks that
/// the range holds and that its name is unique.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tag {
    name: TagName,
    first: u16,
    last: u16,
    loop_mode: LoopMode,
}

impl Tag {
    /// The tag `name` over the frame positions of `range`, played as `loop_mode`.
    #[must_use]
    pub fn new(name: TagName, range: RangeInclusive<u16>, loop_mode: LoopMode) -> Self {
        Self {
            name,
            first: *range.start(),
            last: *range.end(),
            loop_mode,
        }
    }

    /// The tag's name.
    #[must_use]
    pub fn name(&self) -> &TagName {
        &self.name
    }

    /// The position of its first frame.
    #[must_use]
    pub fn first(&self) -> u16 {
        self.first
    }

    /// The position of its last frame, included.
    #[must_use]
    pub fn last(&self) -> u16 {
        self.last
    }

    /// How it plays at its end.
    #[must_use]
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// Whether its range is ordered and inside `frame_count` frames.
    #[must_use]
    pub fn fits(&self, frame_count: usize) -> bool {
        self.first <= self.last && usize::from(self.last) < frame_count
    }
}
