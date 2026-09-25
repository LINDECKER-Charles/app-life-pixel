use serde::{Deserialize, Serialize};

/// A rectangle of pixels, from its top-left corner; it may reach outside the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    /// The column of its left edge.
    pub x: i32,
    /// The row of its top edge.
    pub y: i32,
    /// Its width, in pixels.
    pub width: u32,
    /// Its height, in pixels.
    pub height: u32,
}
