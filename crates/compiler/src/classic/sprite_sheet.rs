//! Sprite sheet: every frame of the range in one indexed PNG, and a JSON saying where.

mod grid;
mod json;

use life_pixel_core::Animation;

use super::ClassicOptions;
use super::indexed_png::{IndexedImage, PNG_MEDIA_TYPE};
use super::plan::Plan;
use crate::{ExportError, ExportFile};
use grid::Grid;

/// The media type of the sheet's description.
const JSON_MEDIA_TYPE: &str = "application/json";

/// The range's frames in a grid of `ceil(√n)` columns, row by row, without padding, as
/// `<stem>.png`; then `<stem>.json`, in Aseprite's "array" layout, with each frame's rectangle
/// and duration, and the tags the range holds.
///
/// # Errors
///
/// [`ExportError::Scale`], [`ExportError::TagNotFound`] or [`ExportError::TooLarge`] when
/// `options` do not fit the animation — the sheet as a whole must fit
/// [`EXPORT_MAX_SIDE`](life_pixel_core::limits::EXPORT_MAX_SIDE).
pub fn export_sprite_sheet(
    animation: &Animation,
    options: &ClassicOptions,
) -> Result<Vec<ExportFile>, ExportError> {
    let plan = Plan::new(animation, options)?;
    let grid = Grid::new(plan.range.frames.len(), plan.width, plan.height)?;
    let sheet = draw(&plan, &grid);
    let sheet_image = IndexedImage {
        palette: plan.palette(),
        width: grid.width,
        height: grid.height,
    };
    let stem = plan.stem();
    let image = ExportFile {
        name: format!("{stem}.png"),
        media_type: PNG_MEDIA_TYPE,
        bytes: sheet_image.encode(&sheet)?,
    };
    let description = ExportFile {
        name: format!("{stem}.json"),
        media_type: JSON_MEDIA_TYPE,
        bytes: json::describe(&plan, &grid, &image.name)?,
    };
    Ok(vec![image, description])
}

/// The sheet's palette indices: each frame in its cell, index 0 where no frame lies.
fn draw(plan: &Plan, grid: &Grid) -> Vec<u8> {
    let sheet_width = usize_of(grid.width);
    let cell_width = usize_of(grid.cell_width);
    let mut sheet = vec![0; sheet_width * usize_of(grid.height)];
    for (position, frame) in plan.range.frames.iter().enumerate() {
        let (left, top) = grid.origin(position);
        let rows = plan.render(frame);
        for (row, pixels) in rows.chunks(cell_width.max(1)).enumerate() {
            let start = (usize_of(top) + row) * sheet_width + usize_of(left);
            sheet[start..start + pixels.len()].copy_from_slice(pixels);
        }
    }
    sheet
}

/// A side in pixels as an index, bounded by `EXPORT_MAX_SIDE` so always representable.
fn usize_of(side: u32) -> usize {
    usize::try_from(side).unwrap_or(usize::MAX)
}
