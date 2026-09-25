//! The operations of `draw`, `edit_frames` and `set_tags` as the tools' schemas describe them,
//! each turned into the use case's own type.

use life_pixel_core::LoopMode;
use life_pixel_core::edit::TagSpec;
use life_pixel_core::limits::{
    MAX_FRAME_DURATION_MS, MAX_FRAMES, MAX_PALETTE_ENTRIES, MIN_FRAME_DURATION_MS,
    TAG_NAME_MAX_CHARS,
};
use life_pixel_service::animation::{DrawOperation, FrameEdit};
use schemars::JsonSchema;
use serde::Deserialize;

/// A drawing operation, tagged by `op`; `from` and `to` are `[x, y]`, and `index` a palette
/// index, 0 being transparent. What falls outside the canvas is clipped.
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum DrawOperationArgument {
    /// Paints one pixel.
    Pixel {
        /// Its column, from 0 at the left.
        x: i32,
        /// Its row, from 0 at the top.
        y: i32,
        /// The palette index painted.
        #[schemars(range(max = MAX_PALETTE_ENTRIES - 1))]
        index: u32,
    },
    /// Paints a line, both ends included.
    Line {
        /// One end.
        from: [i32; 2],
        /// The other end.
        to: [i32; 2],
        /// The palette index painted.
        #[schemars(range(max = MAX_PALETTE_ENTRIES - 1))]
        index: u32,
    },
    /// Paints a rectangle between two opposite corners, outlined unless `filled`.
    Rectangle {
        /// One corner.
        from: [i32; 2],
        /// The opposite corner.
        to: [i32; 2],
        /// The palette index painted.
        #[schemars(range(max = MAX_PALETTE_ENTRIES - 1))]
        index: u32,
        /// Whether the inside is painted too.
        #[serde(default)]
        filled: bool,
    },
    /// Flood-fills the 4-connected area of the layer's own pixels around a point.
    Fill {
        /// The column it starts from.
        x: i32,
        /// The row it starts from.
        y: i32,
        /// The palette index filled with.
        #[schemars(range(max = MAX_PALETTE_ENTRIES - 1))]
        index: u32,
    },
}

/// A frame edit, tagged by `op`. Frames are positions from 0, read after the edits before it.
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum FrameEditArgument {
    /// Inserts a transparent frame.
    Add {
        /// Its position; after the last frame when omitted.
        #[serde(default)]
        #[schemars(range(max = MAX_FRAMES - 1))]
        position: Option<u16>,
        /// How long it shows, in milliseconds; the default duration when omitted.
        #[serde(default)]
        #[schemars(range(min = MIN_FRAME_DURATION_MS, max = MAX_FRAME_DURATION_MS))]
        duration_ms: Option<u32>,
    },
    /// Inserts a copy of a frame right after it.
    Duplicate {
        /// The frame copied.
        #[schemars(range(max = MAX_FRAMES - 1))]
        frame: u16,
    },
    /// Deletes a frame; an animation keeps at least one.
    Delete {
        /// The frame deleted.
        #[schemars(range(max = MAX_FRAMES - 1))]
        frame: u16,
    },
    /// Moves a frame in play order.
    Move {
        /// The frame moved.
        #[schemars(range(max = MAX_FRAMES - 1))]
        frame: u16,
        /// Its new position.
        #[schemars(range(max = MAX_FRAMES - 1))]
        position: u16,
    },
    /// Changes how long a frame shows.
    SetDuration {
        /// The frame changed.
        #[schemars(range(max = MAX_FRAMES - 1))]
        frame: u16,
        /// How long it shows, in milliseconds.
        #[schemars(range(min = MIN_FRAME_DURATION_MS, max = MAX_FRAME_DURATION_MS))]
        duration_ms: u32,
    },
}

/// A tag: a named range of frames, both ends included, and how it plays at its end.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct TagArgument {
    /// Its name, unique in the animation.
    #[schemars(length(max = TAG_NAME_MAX_CHARS))]
    name: String,
    /// The position of its first frame, from 0.
    #[schemars(range(max = MAX_FRAMES - 1))]
    first: u32,
    /// The position of its last frame, included.
    #[schemars(range(max = MAX_FRAMES - 1))]
    last: u32,
    /// `loop` goes back to the first frame; `once` stops on the last.
    #[serde(rename = "loop")]
    loop_mode: LoopArgument,
}

/// How a tag plays at its end.
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum LoopArgument {
    /// Goes back to the tag's first frame.
    Loop,
    /// Stays on the tag's last frame.
    Once,
}

impl From<DrawOperationArgument> for DrawOperation {
    fn from(operation: DrawOperationArgument) -> Self {
        match operation {
            DrawOperationArgument::Pixel { x, y, index } => Self::Pixel { x, y, index },
            DrawOperationArgument::Line { from, to, index } => Self::Line { from, to, index },
            DrawOperationArgument::Rectangle {
                from,
                to,
                index,
                filled,
            } => Self::Rectangle {
                from,
                to,
                index,
                filled,
            },
            DrawOperationArgument::Fill { x, y, index } => Self::Fill { x, y, index },
        }
    }
}

impl From<FrameEditArgument> for FrameEdit {
    fn from(edit: FrameEditArgument) -> Self {
        match edit {
            FrameEditArgument::Add {
                position,
                duration_ms,
            } => Self::Add {
                position,
                duration_ms,
            },
            FrameEditArgument::Duplicate { frame } => Self::Duplicate { frame },
            FrameEditArgument::Delete { frame } => Self::Delete { frame },
            FrameEditArgument::Move { frame, position } => Self::Move { frame, position },
            FrameEditArgument::SetDuration { frame, duration_ms } => {
                Self::SetDuration { frame, duration_ms }
            }
        }
    }
}

impl From<TagArgument> for TagSpec {
    fn from(tag: TagArgument) -> Self {
        let loop_mode = match tag.loop_mode {
            LoopArgument::Loop => LoopMode::Loop,
            LoopArgument::Once => LoopMode::Once,
        };
        Self {
            name: tag.name,
            first: tag.first,
            last: tag.last,
            loop_mode,
        }
    }
}
