//! Where frames lie on a preview, and at which scale: one frame alone, or a contact sheet of
//! `ceil(√n)` columns, row by row, with a transparent gap between cells.

use life_pixel_compiler::ExportError;
use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MIN_SCALE, PREVIEW_MAX_SIDE};

use super::PreviewCell;
use crate::animation::EditingError;

/// The transparent gap between two cells of a contact sheet, before scaling.
const GAP: u32 = 2;
/// The longest side the automatic scale aims for.
const AUTO_SIDE: u32 = 512;
/// The largest automatic scale.
const AUTO_MAX_SCALE: u32 = 8;

/// The grid of a preview before scaling: `columns` cells of a frame's size, `GAP` apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Layout {
    columns: u32,
    cell_width: u32,
    cell_height: u32,
    /// The image's width before scaling.
    pub(super) width: u32,
    /// The image's height before scaling.
    pub(super) height: u32,
}

impl Layout {
    /// The grid of `count` frames of `cell_width × cell_height` pixels: `ceil(√count)` columns.
    pub(super) fn new(count: usize, cell_width: u32, cell_height: u32) -> Self {
        let count = u32::try_from(count).unwrap_or(u32::MAX).max(1);
        let columns = ceil_sqrt(count);
        let rows = count.div_ceil(columns);
        Self {
            columns,
            cell_width,
            cell_height,
            width: span(columns, cell_width),
            height: span(rows, cell_height),
        }
    }

    /// The scale asked for, checked against the export's bounds, or else the largest up to
    /// `AUTO_MAX_SCALE` keeping the longest side within `AUTO_SIDE`, at least 1; either way the
    /// scaled image must fit [`PREVIEW_MAX_SIDE`].
    pub(super) fn scale(&self, asked: Option<u8>) -> Result<u32, EditingError> {
        let longest = self.width.max(self.height).max(1);
        let scale = match asked.map(u32::from) {
            Some(scale) if (EXPORT_MIN_SCALE..=EXPORT_MAX_SCALE).contains(&scale) => scale,
            Some(_) => return Err(ExportError::Scale.into()),
            None => (AUTO_SIDE / longest).clamp(1, AUTO_MAX_SCALE),
        };
        let fits = longest.saturating_mul(scale) <= PREVIEW_MAX_SIDE;
        fits.then_some(scale).ok_or(EditingError::PreviewTooLarge)
    }

    /// The top left corner of the cell at `slot`, before scaling.
    pub(super) fn origin(&self, slot: usize) -> (u32, u32) {
        let slot = u32::try_from(slot).unwrap_or(u32::MAX);
        let column = slot % self.columns;
        let row = slot / self.columns;
        (
            column * (self.cell_width + GAP),
            row * (self.cell_height + GAP),
        )
    }

    /// Where each of the frames at `positions` lies once scaled by `scale`, slot by slot.
    pub(super) fn cells(&self, positions: &[u16], scale: u32) -> Vec<PreviewCell> {
        let slots = positions.iter().enumerate();
        slots
            .map(|(slot, &frame)| {
                let (x, y) = self.origin(slot);
                PreviewCell {
                    frame,
                    x: x * scale,
                    y: y * scale,
                    width: self.cell_width * scale,
                    height: self.cell_height * scale,
                }
            })
            .collect()
    }
}

/// `count` cells of `side` pixels, `GAP` apart.
fn span(count: u32, side: u32) -> u32 {
    count
        .saturating_mul(side)
        .saturating_add(count.saturating_sub(1).saturating_mul(GAP))
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
    fn a_sheet_has_the_ceiling_of_the_square_root_as_columns() {
        let columns = [1, 2, 4, 5, 9, 10].map(|count| Layout::new(count, 1, 1).columns);
        assert_eq!(columns, [1, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn cells_are_a_gap_apart() {
        let layout = Layout::new(5, 4, 3);
        assert_eq!(
            (layout.width, layout.height),
            (3 * 4 + 2 * GAP, 2 * 3 + GAP)
        );
        assert_eq!(layout.origin(2), (2 * (4 + GAP), 0));
        assert_eq!(layout.origin(3), (0, 3 + GAP));
    }

    #[test]
    fn the_automatic_scale_aims_for_512_pixels_up_to_8() {
        assert_eq!(Layout::new(1, 16, 16).scale(None), Ok(8));
        assert_eq!(Layout::new(1, 100, 30).scale(None), Ok(5));
        assert_eq!(Layout::new(1, 512, 512).scale(None), Ok(1));
        assert_eq!(Layout::new(4, 511, 511).scale(None), Ok(1));
        assert_eq!(
            Layout::new(4, 512, 512).scale(None),
            Err(EditingError::PreviewTooLarge)
        );
    }
}
