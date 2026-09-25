//! The pixel operations: each produces the whole cel of one layer on one frame, which `apply`
//! stores and `preview` returns.

mod canvas;
mod fill;
mod line;
mod rectangle;
mod selection;

pub(crate) use canvas::Canvas;

use super::references::{frame_position, layer_position, palette_index};
use super::{Area, EditError, Operation, import};
use crate::limits::STROKE_MAX_POINTS;
use crate::model::{Animation, Cel, FrameId, LayerId, Point};
use rectangle::Rectangle;

/// What a pixel operation produces: the layer and frame it edits, and their new cel.
pub(crate) type EditedCel = (LayerId, FrameId, Cel);

/// The index of a pixel operation without a colour of its own: transparent, in every palette.
const TRANSPARENT_INDEX: u32 = 0;

/// Where a pixel operation paints, and with which palette index.
pub(crate) struct Brush {
    pub(crate) layer: LayerId,
    pub(crate) frame: FrameId,
    pub(crate) index: u32,
}

impl Brush {
    /// The brush of `operation`, when it is a pixel operation.
    fn of(operation: &Operation) -> Option<Self> {
        let (layer, frame) = match operation {
            Operation::PaintStroke { layer, frame, .. }
            | Operation::Fill { layer, frame, .. }
            | Operation::Line { layer, frame, .. }
            | Operation::Rectangle { layer, frame, .. }
            | Operation::MoveSelection { layer, frame, .. }
            | Operation::ImportImage { layer, frame, .. } => (*layer, *frame),
            _ => return None,
        };
        let index = match operation {
            Operation::PaintStroke { index, .. }
            | Operation::Fill { index, .. }
            | Operation::Line { index, .. }
            | Operation::Rectangle { index, .. } => *index,
            _ => TRANSPARENT_INDEX,
        };
        Some(Self {
            layer,
            frame,
            index,
        })
    }
}

/// The cel `operation` produces, when it is a pixel operation.
pub(crate) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<EditedCel, EditError>> {
    let brush = Brush::of(operation)?;
    let edited = match operation {
        Operation::PaintStroke { points, .. } => stroke(animation, brush, points),
        Operation::Fill { at, .. } => fill(animation, brush, *at),
        Operation::Line { from, to, .. } => line(animation, brush, (*from, *to)),
        Operation::Rectangle {
            from, to, filled, ..
        } => {
            let rectangle = Rectangle::between(*from, *to, *filled);
            paint(animation, brush, |canvas, index| {
                rectangle.draw(canvas, index);
                Ok(())
            })
        }
        Operation::MoveSelection { area, offset, .. } => {
            move_selection(animation, brush, (*area, *offset))
        }
        Operation::ImportImage { png, at, .. } => import_image(animation, brush, (png, *at)),
        _ => return None,
    };
    Some(edited)
}

/// Checks the brush's layer, frame and index, in that order, then draws on a copy of its cel.
pub(crate) fn paint(
    animation: &Animation,
    brush: Brush,
    draw: impl FnOnce(&mut Canvas, u8) -> Result<(), EditError>,
) -> Result<EditedCel, EditError> {
    layer_position(animation, brush.layer)?;
    frame_position(animation, brush.frame)?;
    let index = palette_index(animation, brush.index)?;
    let mut canvas = Canvas::of(animation, brush.layer, brush.frame);
    draw(&mut canvas, index)?;
    Ok((brush.layer, brush.frame, canvas.into_cel()))
}

/// At most [`STROKE_MAX_POINTS`] points.
fn stroke(animation: &Animation, brush: Brush, points: &[Point]) -> Result<EditedCel, EditError> {
    paint(animation, brush, |canvas, index| {
        if points.len() > STROKE_MAX_POINTS {
            return Err(EditError::StrokeTooLong);
        }
        line::draw_stroke(canvas, points, index);
        Ok(())
    })
}

/// `at` must lie on the canvas.
fn fill(animation: &Animation, brush: Brush, at: Point) -> Result<EditedCel, EditError> {
    paint(animation, brush, |canvas, index| {
        if !canvas.contains(at) {
            return Err(EditError::OutOfCanvas);
        }
        fill::flood_fill(canvas, at, index);
        Ok(())
    })
}

fn line(animation: &Animation, brush: Brush, ends: (Point, Point)) -> Result<EditedCel, EditError> {
    paint(animation, brush, |canvas, index| {
        line::draw_line(canvas, ends, index);
        Ok(())
    })
}

fn move_selection(
    animation: &Animation,
    brush: Brush,
    (area, offset): (Area, Point),
) -> Result<EditedCel, EditError> {
    paint(animation, brush, |canvas, _| {
        selection::move_selection(canvas, area, offset);
        Ok(())
    })
}

/// Pastes at `at`, clipped; pixels reduced to 0 leave the cel as it was.
fn import_image(
    animation: &Animation,
    brush: Brush,
    (png, at): (&[u8], Point),
) -> Result<EditedCel, EditError> {
    paint(animation, brush, |canvas, _| {
        let image = import::indexed(png, animation.palette())?;
        image.paste_whole(canvas, at);
        Ok(())
    })
}
