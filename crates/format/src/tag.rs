use crate::LoopMode;

/// A named range of frames, borrowed from a parsed payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag<'a> {
    /// The name: 1 to [`MAX_TAG_NAME_BYTES`](crate::bounds::MAX_TAG_NAME_BYTES) bytes of UTF-8.
    pub name: &'a str,
    /// The index of the range's first frame.
    pub first: u16,
    /// The index of the range's last frame, included: `first ≤ last < frame count`.
    pub last: u16,
    /// How the range plays when it reaches `last`.
    pub loop_mode: LoopMode,
}
