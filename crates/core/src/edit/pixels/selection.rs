use super::canvas::Canvas;
use crate::edit::Area;
use crate::model::Point;

/// Moves the pixels of `area`, clipped to the canvas, by `offset`: the area is cleared to 0, then
/// its non-zero pixels are pasted; those leaving the canvas are lost.
pub(crate) fn move_selection(canvas: &mut Canvas, area: Area, offset: Point) {
    let pixels = take_area(canvas, area);
    let (step_x, step_y) = (i64::from(offset.x), i64::from(offset.y));
    for (x, y, index) in pixels {
        canvas.set((x + step_x, y + step_y), index);
    }
}

/// Clears the part of `area` on the canvas to 0, and returns its non-zero pixels.
fn take_area(canvas: &mut Canvas, area: Area) -> Vec<(i64, i64, u8)> {
    let (left, top) = (i64::from(area.x), i64::from(area.y));
    let columns = left.max(0)..(left + i64::from(area.width)).min(canvas.width());
    let rows = top.max(0)..(top + i64::from(area.height)).min(canvas.height());
    let mut pixels = Vec::new();
    for y in rows {
        for x in columns.clone() {
            let index = canvas.get((x, y)).unwrap_or_default();
            if index != 0 {
                pixels.push((x, y, index));
                canvas.set((x, y), 0);
            }
        }
    }
    pixels
}
