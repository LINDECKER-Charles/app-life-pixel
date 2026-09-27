use serde::{Deserialize, Serialize};

/// A pixel position, from the top left of the canvas; it may lie outside the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    /// The column, growing to the right.
    pub x: i32,
    /// The row, growing downwards.
    pub y: i32,
}
