//! Where each frame lies on the sheet.

use crate::ExportError;
use crate::classic::scale::bounded_side;

/// A grid of `ceil(√n)` columns and as many rows as the frames need, filled row by row.
pub(super) struct Grid {
    /// The number of columns.
    pub columns: u32,
    /// A cell's width: a frame's, once scaled.
    pub cell_width: u32,
    /// A cell's height: a frame's, once scaled.
    pub cell_height: u32,
    /// The sheet's width.
    pub width: u32,
    /// The sheet's height.
    pub height: u32,
}

impl Grid {
    /// The grid of `frame_count` cells of `cell_width × cell_height` pixels, when the sheet fits
    /// [`EXPORT_MAX_SIDE`](life_pixel_core::limits::EXPORT_MAX_SIDE).
    pub fn new(frame_count: usize, cell_width: u32, cell_height: u32) -> Result<Self, ExportError> {
        let frame_count = u32::try_from(frame_count).map_err(|_| ExportError::TooLarge)?;
        let columns = ceil_sqrt(frame_count).max(1);
        let rows = frame_count.div_ceil(columns).max(1);
        Ok(Self {
            columns,
            cell_width,
            cell_height,
            width: bounded_side(cell_width, columns)?,
            height: bounded_side(cell_height, rows)?,
        })
    }

    /// The top left corner of the cell of the frame at `position` in the range.
    pub fn origin(&self, position: usize) -> (u32, u32) {
        let position = u32::try_from(position).unwrap_or(u32::MAX);
        let column = position % self.columns;
        let row = position / self.columns;
        (column * self.cell_width, row * self.cell_height)
    }
}

/// The smallest `root` with `root × root ≥ value`.
fn ceil_sqrt(value: u32) -> u32 {
    let root = value.isqrt();
    if root * root < value { root + 1 } else { root }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_are_the_ceiling_of_the_square_root() {
        let columns = [1, 2, 3, 4, 5, 9, 10].map(|count| Grid::new(count, 1, 1).unwrap().columns);
        assert_eq!(columns, [1, 2, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn frames_fill_the_grid_row_by_row() {
        let grid = Grid::new(5, 4, 3).unwrap();
        assert_eq!((grid.width, grid.height), (12, 6));
        assert_eq!(grid.origin(0), (0, 0));
        assert_eq!(grid.origin(2), (8, 0));
        assert_eq!(grid.origin(3), (0, 3));
    }
}
