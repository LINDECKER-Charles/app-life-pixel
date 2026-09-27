use crate::FrameKind;

/// A frame of a parsed payload: its duration and its encoded operations, not yet applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    /// How long the frame shows, in milliseconds: at least 1.
    pub duration_ms: u16,
    /// Whether the operations apply to a blank canvas or to the previous frame.
    pub kind: FrameKind,
    /// The operations, as [`apply_frame`](crate::apply_frame) reads them.
    pub data: &'a [u8],
}
