use alloc::string::String;

use crate::LoopMode;

/// A tag to encode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagData {
    /// The name, 1 to [`MAX_TAG_NAME_BYTES`](crate::bounds::MAX_TAG_NAME_BYTES) bytes.
    pub name: String,
    /// The index of the range's first frame.
    pub first: u16,
    /// The index of the range's last frame, included.
    pub last: u16,
    /// How the range plays when it reaches `last`.
    pub loop_mode: LoopMode,
}
