//! Drawing on a cel: an agent's operations as `core`'s pixel operations.

use life_pixel_core::edit::Operation;
use life_pixel_core::{Animation, FrameId, LayerId, Point};
use serde::{Deserialize, Serialize};

use crate::animation::EditingError;
use crate::animation::addressing::{Target, apply_all};

/// One drawing operation of `draw`, tagged by `op` as the MCP tool takes it; points are `[x, y]`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DrawOperation {
    /// One pixel, `core`'s pencil on one point; outside the canvas it paints nothing.
    Pixel {
        /// Its column.
        x: i32,
        /// Its row.
        y: i32,
        /// The palette index painted.
        index: u32,
    },
    /// A line, both ends included, clipped to the canvas.
    Line {
        /// One end.
        from: [i32; 2],
        /// The other end.
        to: [i32; 2],
        /// The palette index painted.
        index: u32,
    },
    /// A rectangle between two corners, outlined unless `filled`, clipped to the canvas.
    Rectangle {
        /// One corner.
        from: [i32; 2],
        /// The opposite corner.
        to: [i32; 2],
        /// The palette index painted.
        index: u32,
        /// Whether the inside is painted too.
        #[serde(default)]
        filled: bool,
    },
    /// A 4-connected flood fill of the layer's own cel, from a point on the canvas.
    Fill {
        /// The column it starts from.
        x: i32,
        /// The row it starts from.
        y: i32,
        /// The palette index filled with.
        index: u32,
    },
}

impl DrawOperation {
    /// The `core` operation that draws this on `layer` and `frame`.
    fn to_operation(&self, (layer, frame): (LayerId, FrameId)) -> Operation {
        match *self {
            Self::Pixel { x, y, index } => Operation::PaintStroke {
                layer,
                frame,
                points: vec![Point { x, y }],
                index,
            },
            Self::Line { from, to, index } => Operation::Line {
                layer,
                frame,
                from: point(from),
                to: point(to),
                index,
            },
            Self::Rectangle {
                from,
                to,
                index,
                filled,
            } => rectangle((layer, frame), (point(from), point(to)), (index, filled)),
            Self::Fill { x, y, index } => Operation::Fill {
                layer,
                frame,
                at: Point { x, y },
                index,
            },
        }
    }
}

/// A rectangle's operation: its cel, its corners, its index and whether it is filled.
fn rectangle(
    (layer, frame): (LayerId, FrameId),
    (from, to): (Point, Point),
    (index, filled): (u32, bool),
) -> Operation {
    Operation::Rectangle {
        layer,
        frame,
        from,
        to,
        index,
        filled,
    }
}

/// Applies `operations` to the layer and frame of `target`.
pub(super) fn draw(
    animation: &mut Animation,
    target: Target,
    operations: &[DrawOperation],
) -> Result<(), EditingError> {
    let cel = target.resolve(animation)?;
    let operations: Vec<Operation> = operations
        .iter()
        .map(|operation| operation.to_operation(cel))
        .collect();
    Ok(apply_all(animation, &operations)?)
}

fn point([x, y]: [i32; 2]) -> Point {
    Point { x, y }
}
