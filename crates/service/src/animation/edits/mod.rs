//! The changes of agents: palette, text grids, drawing, frames and tags, each applied by `core`
//! in [`AnimationEditing::change`], which saves the result.

mod drawing;
mod frames;
mod grid_cel;
mod palette;

use std::sync::Arc;

pub use drawing::DrawOperation;
pub use frames::FrameEdit;
use life_pixel_core::Rgba;
use life_pixel_core::edit::{Operation, TagSpec};
use life_pixel_core::limits::DRAW_MAX_OPERATIONS;

use super::addressing::{Target, apply_all};
use super::{AnimationEditing, AnimationView, EditingError};
use crate::ids::AnimationId;
use crate::owner::Owner;

/// The new palette of an animation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetPaletteRequest {
    /// The animation.
    pub id: AnimationId,
    /// Every entry, entry 0 first: it must be transparent, and entry 0 keeps its own colour.
    pub colors: Vec<Rgba>,
}

/// A text grid to write on one layer of one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteFrameRequest {
    /// The animation.
    pub id: AnimationId,
    /// The frame's position, from 0.
    pub frame: u16,
    /// The layer's id; `None` for the top layer.
    pub layer: Option<u32>,
    /// The rows, top to bottom, in the grid alphabet of `core`.
    pub grid: Vec<String>,
}

/// A batch of drawing operations on one layer of one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawRequest {
    /// The animation.
    pub id: AnimationId,
    /// The frame's position, from 0.
    pub frame: u16,
    /// The layer's id; `None` for the top layer.
    pub layer: Option<u32>,
    /// The operations, in drawing order; at most [`DRAW_MAX_OPERATIONS`].
    pub operations: Vec<DrawOperation>,
}

/// Frame edits, applied in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditFramesRequest {
    /// The animation.
    pub id: AnimationId,
    /// The edits; each addresses frames by their positions after the edits before it.
    pub edits: Vec<FrameEdit>,
}

/// The tags that replace every tag of an animation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetTagsRequest {
    /// The animation.
    pub id: AnimationId,
    /// The new tags.
    pub tags: Vec<TagSpec>,
}

impl AnimationEditing {
    /// Replaces the palette. Entries are positional: a pixel keeps its index, so an entry whose
    /// colour changes repaints its pixels.
    ///
    /// # Errors
    ///
    /// `document.palette` when entry 0 is not transparent or the entries are 0 or above 256;
    /// `edit.palette_in_use` when a dropped entry is still painted; the library's codes.
    pub async fn set_palette(
        &self,
        owner: &Owner,
        request: SetPaletteRequest,
    ) -> Result<AnimationView, EditingError> {
        let colors = request.colors;
        let edit = Arc::new(move |animation: &mut _| palette::set_palette(animation, &colors));
        self.change(owner, (request.id, edit)).await
    }

    /// Replaces the cel of a layer on a frame with a text grid.
    ///
    /// # Errors
    ///
    /// `grid.size`, `grid.character` or `grid.index` for a grid that does not fit;
    /// `edit.frame_not_found`; `edit.layer_not_found`; the library's codes.
    pub async fn write_frame(
        &self,
        owner: &Owner,
        request: WriteFrameRequest,
    ) -> Result<AnimationView, EditingError> {
        let target = Target {
            frame: request.frame,
            layer: request.layer,
        };
        let rows = request.grid;
        let edit =
            Arc::new(move |animation: &mut _| grid_cel::write_grid(animation, target, &rows));
        self.change(owner, (request.id, edit)).await
    }

    /// Draws pixels, lines, rectangles and fills on a layer of a frame, in order.
    ///
    /// # Errors
    ///
    /// `draw.too_many_operations`; the `edit.*` code of the first operation `core` refuses,
    /// which leaves the animation unchanged; the library's codes.
    pub async fn draw(
        &self,
        owner: &Owner,
        request: DrawRequest,
    ) -> Result<AnimationView, EditingError> {
        if request.operations.len() > DRAW_MAX_OPERATIONS {
            return Err(EditingError::TooManyOperations);
        }
        let target = Target {
            frame: request.frame,
            layer: request.layer,
        };
        let operations = request.operations;
        let edit = Arc::new(move |animation: &mut _| drawing::draw(animation, target, &operations));
        self.change(owner, (request.id, edit)).await
    }

    /// Adds, duplicates, deletes and moves frames, and sets their durations, in order; tags
    /// follow as `core` moves them.
    ///
    /// # Errors
    ///
    /// The `edit.*` or `document.*` code of the first edit `core` refuses, which leaves the
    /// animation unchanged; the library's codes.
    pub async fn edit_frames(
        &self,
        owner: &Owner,
        request: EditFramesRequest,
    ) -> Result<AnimationView, EditingError> {
        let edits = request.edits;
        let edit = Arc::new(move |animation: &mut _| frames::edit_frames(animation, &edits));
        self.change(owner, (request.id, edit)).await
    }

    /// Replaces every tag at once, through `core`'s `replaceTags`.
    ///
    /// # Errors
    ///
    /// `document.tag` or `document.tag_count` for tags that break the model's rules; the
    /// library's codes.
    pub async fn set_tags(
        &self,
        owner: &Owner,
        request: SetTagsRequest,
    ) -> Result<AnimationView, EditingError> {
        let operation = [Operation::ReplaceTags { tags: request.tags }];
        let edit = Arc::new(move |animation: &mut _| Ok(apply_all(animation, &operation)?));
        self.change(owner, (request.id, edit)).await
    }
}
