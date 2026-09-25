mod area;
mod bytes;
mod tag_spec;

pub use area::Area;
use serde::{Deserialize, Serialize};
pub use tag_spec::TagSpec;

use crate::model::{FrameId, LayerId, Point, Rgba};

/// One edit of an animation: a tool stroke, a palette, layer, frame or tag change, an import.
///
/// Serialized with a `kind` in camelCase and camelCase fields — the shape the editor's engine
/// interface mirrors field for field. Numbers are read wider than the model holds them, so that
/// an out-of-range value fails with its own code when applied rather than as malformed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Operation {
    /// The pencil, or the eraser with index 0: each point, and a line between consecutive points.
    /// Pixels outside the canvas are ignored.
    PaintStroke {
        /// The layer drawn on.
        layer: LayerId,
        /// The frame drawn on.
        frame: FrameId,
        /// The points, in drawing order; at most `STROKE_MAX_POINTS`.
        points: Vec<Point>,
        /// The palette index painted.
        index: u32,
    },
    /// A 4-connected flood fill of the layer's own cel, not of the composite.
    Fill {
        /// The layer filled.
        layer: LayerId,
        /// The frame filled.
        frame: FrameId,
        /// Where the fill starts; it must lie on the canvas.
        at: Point,
        /// The palette index filled with.
        index: u32,
    },
    /// A line, both ends included, clipped to the canvas.
    Line {
        /// The layer drawn on.
        layer: LayerId,
        /// The frame drawn on.
        frame: FrameId,
        /// One end.
        from: Point,
        /// The other end.
        to: Point,
        /// The palette index painted.
        index: u32,
    },
    /// A rectangle between two corners given in any order, both included, clipped to the canvas.
    Rectangle {
        /// The layer drawn on.
        layer: LayerId,
        /// The frame drawn on.
        frame: FrameId,
        /// One corner.
        from: Point,
        /// The opposite corner.
        to: Point,
        /// The palette index painted.
        index: u32,
        /// Whether the inside is painted too, or only a one-pixel outline.
        filled: bool,
    },
    /// Moves the pixels of `area`, clipped to the canvas, by `offset`: the area is cleared to 0,
    /// then its pixels are pasted, index 0 leaving the destination as it was.
    MoveSelection {
        /// The layer edited.
        layer: LayerId,
        /// The frame edited.
        frame: FrameId,
        /// The pixels moved.
        area: Area,
        /// How far they move.
        offset: Point,
    },
    /// Changes the colour of a palette entry other than 0.
    SetPaletteEntry {
        /// The entry changed, 1 or above.
        index: u32,
        /// Its new colour.
        color: Rgba,
    },
    /// Appends a palette entry.
    AddPaletteEntry {
        /// Its colour.
        color: Rgba,
    },
    /// Removes a palette entry other than 0: its pixels become 0, higher indices move down by one.
    RemovePaletteEntry {
        /// The entry removed, 1 or above.
        index: u32,
    },
    /// Moves a palette entry other than 0 to another position, every cel remapped.
    MovePaletteEntry {
        /// Its position, 1 or above.
        from: u32,
        /// Its new position, 1 or above.
        to: u32,
    },
    /// Inserts a new, empty layer.
    AddLayer {
        /// Its position, 0 being the bottom.
        position: u32,
        /// Its name.
        name: String,
    },
    /// Deletes a layer and its cels.
    DeleteLayer {
        /// The layer deleted.
        layer: LayerId,
    },
    /// Moves a layer in the stack.
    MoveLayer {
        /// The layer moved.
        layer: LayerId,
        /// Its new position, 0 being the bottom.
        position: u32,
    },
    /// Renames a layer.
    RenameLayer {
        /// The layer renamed.
        layer: LayerId,
        /// Its new name.
        name: String,
    },
    /// Shows or hides a layer.
    SetLayerVisibility {
        /// The layer changed.
        layer: LayerId,
        /// Whether it is shown.
        visible: bool,
    },
    /// Inserts a new frame with empty cels.
    AddFrame {
        /// Its position, 0 being the first.
        position: u32,
        /// How long it shows, in milliseconds.
        duration_ms: u32,
    },
    /// Inserts a copy of a frame, cels included, right after it.
    DuplicateFrame {
        /// The frame copied.
        frame: FrameId,
    },
    /// Deletes a frame and its cels.
    DeleteFrame {
        /// The frame deleted.
        frame: FrameId,
    },
    /// Moves a frame in play order; tags keep their positions.
    MoveFrame {
        /// The frame moved.
        frame: FrameId,
        /// Its new position, 0 being the first.
        position: u32,
    },
    /// Changes how long a frame shows.
    SetFrameDuration {
        /// The frame changed.
        frame: FrameId,
        /// How long it shows, in milliseconds.
        duration_ms: u32,
    },
    /// Adds a tag.
    AddTag {
        /// The tag added.
        tag: TagSpec,
    },
    /// Replaces the tag of a name.
    UpdateTag {
        /// The name of the tag replaced.
        name: String,
        /// The tag that replaces it.
        tag: TagSpec,
    },
    /// Deletes the tag of a name.
    DeleteTag {
        /// The name of the tag deleted.
        name: String,
    },
    /// Replaces every tag at once.
    ReplaceTags {
        /// The new tags.
        tags: Vec<TagSpec>,
    },
    /// Changes the title.
    SetTitle {
        /// The new title.
        title: String,
    },
    /// Pastes a PNG image, reduced to the palette, on a cel.
    ImportImage {
        /// The layer pasted on.
        layer: LayerId,
        /// The frame pasted on.
        frame: FrameId,
        /// The PNG file.
        #[serde(with = "bytes")]
        png: Vec<u8>,
        /// Where the image's top-left pixel goes.
        at: Point,
    },
    /// Cuts a PNG sprite sheet into cells, row by row, and inserts one frame per cell.
    ImportSpriteSheet {
        /// The layer the cells go on.
        layer: LayerId,
        /// The position of the first new frame.
        position: u32,
        /// The PNG file.
        #[serde(with = "bytes")]
        png: Vec<u8>,
        /// The width of a cell, in pixels.
        cell_width: u32,
        /// The height of a cell, in pixels.
        cell_height: u32,
        /// How long each new frame shows, in milliseconds.
        duration_ms: u32,
    },
}
