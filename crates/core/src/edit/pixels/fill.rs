use super::canvas::Canvas;
use crate::model::Point;

/// The four neighbours of a pixel: right, left, down, up.
const NEIGHBOURS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Fills with `index` the 4-connected region of `at`'s index around `at`. An explicit stack keeps
/// any shape — a spiral, a maze — within bounded memory.
pub(crate) fn flood_fill(canvas: &mut Canvas, at: Point, index: u8) {
    let start = (i64::from(at.x), i64::from(at.y));
    let Some(target) = canvas.get(start) else {
        return;
    };
    if target == index {
        return;
    }
    canvas.set(start, index);
    let mut pending = vec![start];
    while let Some((x, y)) = pending.pop() {
        for (step_x, step_y) in NEIGHBOURS {
            let next = (x + step_x, y + step_y);
            if canvas.get(next) == Some(target) {
                canvas.set(next, index);
                pending.push(next);
            }
        }
    }
}
