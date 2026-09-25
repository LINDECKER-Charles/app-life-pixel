use alloc::vec::Vec;

/// A frame to encode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameData {
    /// How long the frame shows, in milliseconds: at least 1.
    pub duration_ms: u16,
    /// `width × height` palette indices, row by row from the top-left pixel.
    pub indices: Vec<u8>,
}
