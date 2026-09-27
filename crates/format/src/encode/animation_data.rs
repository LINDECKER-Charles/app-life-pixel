use alloc::{string::String, vec::Vec};

use super::{FrameData, TagData};
use crate::Rgba;

/// An animation ready to encode: its layers already flattened into palette indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationData {
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// 1 to 256 entries; entry 0 is transparent.
    pub palette: Vec<Rgba>,
    /// The title, possibly empty.
    pub title: String,
    /// The named ranges of frames.
    pub tags: Vec<TagData>,
    /// The frames, in order.
    pub frames: Vec<FrameData>,
}
