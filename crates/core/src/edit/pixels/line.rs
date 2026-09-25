//! Bresenham lines, both ends included. Only the steps whose major coordinate lies on the canvas
//! are walked, so a line between far-away points costs no more than one across the canvas.

use super::canvas::Canvas;
use crate::model::Point;

/// Paints the line between both ends of `segment` with `index`, clipped to the canvas.
pub(crate) fn draw_line(canvas: &mut Canvas, (from, to): (Point, Point), index: u8) {
    let (from_x, from_y) = (i64::from(from.x), i64::from(from.y));
    let delta_x = i64::from(to.x) - from_x;
    let delta_y = i64::from(to.y) - from_y;
    if delta_x.abs() >= delta_y.abs() {
        let axis = Axis {
            start: from_x,
            delta: delta_x,
            minor_start: from_y,
            minor_delta: delta_y,
            side: canvas.width(),
        };
        axis.steps().for_each(|(x, y)| canvas.set((x, y), index));
    } else {
        let axis = Axis {
            start: from_y,
            delta: delta_y,
            minor_start: from_x,
            minor_delta: delta_x,
            side: canvas.height(),
        };
        axis.steps().for_each(|(y, x)| canvas.set((x, y), index));
    }
}

/// Paints each point of a stroke and the line between consecutive points.
pub(crate) fn draw_stroke(canvas: &mut Canvas, points: &[Point], index: u8) {
    if let [point] = points {
        draw_line(canvas, (*point, *point), index);
    }
    for pair in points.windows(2) {
        draw_line(canvas, (pair[0], pair[1]), index);
    }
}

/// A line seen along its major axis: one step per pixel on it, the minor coordinate rounded
/// half up from the exact line.
struct Axis {
    start: i64,
    delta: i64,
    minor_start: i64,
    minor_delta: i64,
    /// The canvas side along the major axis.
    side: i64,
}

impl Axis {
    /// The `(major, minor)` coordinates of the steps whose major coordinate is on the canvas.
    fn steps(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        let (first, last) = self.visible_steps();
        (first..=last).map(|step| {
            let major = self.start + self.delta.signum() * step;
            let minor = self.minor_start + self.minor_delta.signum() * self.minor_offset(step);
            (major, minor)
        })
    }

    /// The first and last steps whose major coordinate lies in `0..side`; none when first > last.
    fn visible_steps(&self) -> (i64, i64) {
        let last_on_canvas = self.side - 1;
        let (low, high) = match self.delta.signum() {
            1 => (-self.start, last_on_canvas - self.start),
            -1 => (self.start - last_on_canvas, self.start),
            _ if (0..self.side).contains(&self.start) => (0, 0),
            _ => (1, 0),
        };
        (low.max(0), high.min(self.delta.abs()))
    }

    /// How far the minor coordinate has moved at `step`: `step × |minor| / |major|`, rounded half
    /// up. Wide arithmetic: both factors may reach 2³².
    fn minor_offset(&self, step: i64) -> i64 {
        let length = i128::from(self.delta.abs());
        if length == 0 {
            return 0;
        }
        let rise = i128::from(self.minor_delta.abs());
        let offset = (2 * i128::from(step) * rise + length) / (2 * length);
        i64::try_from(offset).unwrap_or_default()
    }
}
