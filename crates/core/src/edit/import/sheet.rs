//! A sprite sheet: an image cut into cells, row by row, one new frame per cell. The fully
//! transparent cells at the end are dropped.

use super::indexed_image::{IndexedImage, Placement};
use crate::edit::change::{CelChange, Change};
use crate::edit::pixels::Canvas;
use crate::edit::references::{frame_duration, layer_position, position};
use crate::edit::structure::{new_ids, tags_after_insert};
use crate::edit::{Area, EditError, Operation, import};
use crate::error::DocumentError;
use crate::limits::MAX_FRAMES;
use crate::model::{Animation, Frame, FrameId, LayerId, Point};

/// The change an `importSpriteSheet` operation makes.
pub(crate) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    let Operation::ImportSpriteSheet {
        layer,
        position,
        png,
        cell_width,
        cell_height,
        duration_ms,
    } = operation
    else {
        return None;
    };
    let sheet = SheetImport {
        layer: *layer,
        position: *position,
        png,
        cell_size: (*cell_width, *cell_height),
        duration_ms: *duration_ms,
    };
    Some(sheet.plan(animation))
}

/// The fields of an `importSpriteSheet` operation.
struct SheetImport<'operation> {
    layer: LayerId,
    position: u32,
    png: &'operation [u8],
    cell_size: (u32, u32),
    duration_ms: u32,
}

impl SheetImport<'_> {
    /// Checks the references first, then decodes, cuts and builds the new frames.
    fn plan(&self, animation: &Animation) -> Result<Change, EditError> {
        layer_position(animation, self.layer)?;
        let at = position(self.position, animation.frames().len())?;
        let duration_ms = frame_duration(self.duration_ms)?;
        let image = import::indexed(self.png, animation.palette())?;
        let grid = Grid::of(&image, self.cell_size).ok_or(EditError::SheetGrid)?;
        let (frames, next_id) = new_frames(animation, grid.kept_cells(), duration_ms)?;
        let cels = cels(animation, &grid, (self.layer, &frames))?;
        let new = NewFrames {
            frames,
            next_id,
            cels,
        };
        Ok(inserted(animation, at, new))
    }
}

/// The frames a sheet adds, the `nextId` after them, and their cels.
struct NewFrames {
    frames: Vec<Frame>,
    next_id: u32,
    cels: Vec<CelChange>,
}

/// An image cut into cells of one size, which it holds exactly.
struct Grid<'image> {
    image: &'image IndexedImage,
    columns: u32,
    cell_width: u32,
    cell_height: u32,
    cell_count: u32,
}

impl<'image> Grid<'image> {
    fn of(image: &'image IndexedImage, (cell_width, cell_height): (u32, u32)) -> Option<Self> {
        let is_whole = cell_width > 0
            && cell_height > 0
            && image.width().is_multiple_of(cell_width)
            && image.height().is_multiple_of(cell_height);
        is_whole.then(|| {
            let columns = image.width() / cell_width;
            Self {
                image,
                columns,
                cell_width,
                cell_height,
                cell_count: columns * (image.height() / cell_height),
            }
        })
    }

    /// The area of cell `number`, counted row by row.
    fn cell(&self, number: u32) -> Area {
        let column = number % self.columns;
        let row = number / self.columns;
        Area {
            x: i32::try_from(column * self.cell_width).unwrap_or(i32::MAX),
            y: i32::try_from(row * self.cell_height).unwrap_or(i32::MAX),
            width: self.cell_width,
            height: self.cell_height,
        }
    }

    /// How many cells are left once the fully transparent ones at the end are dropped.
    fn kept_cells(&self) -> u32 {
        (0..self.cell_count)
            .rev()
            .find(|&number| !self.image.is_blank_in(self.cell(number)))
            .map_or(0, |last| last + 1)
    }
}

/// `count` new frames and the `nextId` after them, refused before anything is built when the
/// animation cannot hold them.
fn new_frames(
    animation: &Animation,
    count: u32,
    duration_ms: u16,
) -> Result<(Vec<Frame>, u32), EditError> {
    let count = usize::try_from(count).unwrap_or(usize::MAX);
    if animation.frames().len().saturating_add(count) > MAX_FRAMES {
        return Err(EditError::Document(DocumentError::FrameCount));
    }
    let ids = new_ids(animation, count)?;
    let next_id = ids.end;
    let frames = ids
        .map(|id| Frame::new(FrameId::new(id), duration_ms))
        .collect::<Result<_, _>>()?;
    Ok((frames, next_id))
}

/// The non-blank cel of each new frame on `layer`, the cell pasted at the top left; refused as
/// soon as the pixel budget is exceeded.
fn cels(
    animation: &Animation,
    grid: &Grid,
    (layer, frames): (LayerId, &[Frame]),
) -> Result<Vec<CelChange>, EditError> {
    let mut cels = Vec::new();
    for (number, frame) in (0..).zip(frames) {
        let mut canvas = Canvas::blank(animation);
        let placement = Placement {
            source: grid.cell(number),
            at: Point::default(),
        };
        grid.image.paste(&mut canvas, &placement);
        let cel = canvas.into_cel();
        if !cel.is_blank() {
            animation.check_cel_count(animation.cels().len() + cels.len() + 1)?;
            cels.push(((layer, frame.id()), Some(cel)));
        }
    }
    Ok(cels)
}

/// The frames and tags once `new` is inserted at `at`; no change when there is no new frame.
fn inserted(animation: &Animation, at: usize, new: NewFrames) -> Change {
    if new.frames.is_empty() {
        return Change::default();
    }
    let count = new.frames.len();
    let mut frames = animation.frames().to_vec();
    frames.splice(at..at, new.frames);
    Change {
        frames: Some(frames),
        tags: Some(tags_after_insert(animation.tags(), at, count)),
        next_id: Some(new.next_id),
        cels: new.cels,
        ..Change::default()
    }
}
