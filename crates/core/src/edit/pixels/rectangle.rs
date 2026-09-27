use std::ops::RangeInclusive;

use super::canvas::Canvas;
use crate::model::Point;

/// A rectangle between two corners given in any order, both included: filled, or a one-pixel
/// outline.
pub(crate) struct Rectangle {
    columns: RangeInclusive<i64>,
    rows: RangeInclusive<i64>,
    is_filled: bool,
}

impl Rectangle {
    /// The rectangle with corners `from` and `to`.
    pub(crate) fn between(from: Point, to: Point, is_filled: bool) -> Self {
        let (left, right) = ordered(from.x, to.x);
        let (top, bottom) = ordered(from.y, to.y);
        Self {
            columns: left..=right,
            rows: top..=bottom,
            is_filled,
        }
    }

    /// Paints it on the canvas with `index`, clipped.
    pub(crate) fn draw(&self, canvas: &mut Canvas, index: u8) {
        if self.is_filled {
            self.fill(canvas, index);
        } else {
            self.outline(canvas, index);
        }
    }

    /// Paints every pixel of it on the canvas with `index`.
    fn fill(&self, canvas: &mut Canvas, index: u8) {
        for y in clip(&self.rows, canvas.height()) {
            for x in clip(&self.columns, canvas.width()) {
                canvas.set((x, y), index);
            }
        }
    }

    /// Paints its one-pixel outline on the canvas with `index`.
    fn outline(&self, canvas: &mut Canvas, index: u8) {
        let (top, bottom) = (*self.rows.start(), *self.rows.end());
        let (left, right) = (*self.columns.start(), *self.columns.end());
        for x in clip(&self.columns, canvas.width()) {
            canvas.set((x, top), index);
            canvas.set((x, bottom), index);
        }
        for y in clip(&self.rows, canvas.height()) {
            canvas.set((left, y), index);
            canvas.set((right, y), index);
        }
    }
}

fn ordered(first: i32, second: i32) -> (i64, i64) {
    (i64::from(first.min(second)), i64::from(first.max(second)))
}

/// The part of `range` within `0..side`.
fn clip(range: &RangeInclusive<i64>, side: i64) -> RangeInclusive<i64> {
    (*range.start()).max(0)..=(*range.end()).min(side - 1)
}
