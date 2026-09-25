//! Frame edits by position, as `core`'s frame operations.

use life_pixel_core::Animation;
use life_pixel_core::edit::{self, EditError, Operation};
use life_pixel_core::limits::DEFAULT_FRAME_DURATION_MS;
use serde::{Deserialize, Serialize};

use crate::animation::EditingError;
use crate::animation::addressing::frame_at;

/// One edit of `edit_frames`, tagged by `op` as the MCP tool takes it. Frames are positions from
/// 0; durations are milliseconds, checked by `core`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum FrameEdit {
    /// Inserts an empty frame.
    Add {
        /// Its position; after the last frame when `None`.
        #[serde(default)]
        position: Option<u16>,
        /// How long it shows; `DEFAULT_FRAME_DURATION_MS` when `None`.
        #[serde(default)]
        duration_ms: Option<u32>,
    },
    /// Inserts a copy of a frame, cels included, right after it.
    Duplicate {
        /// The frame copied.
        frame: u16,
    },
    /// Deletes a frame and its cels.
    Delete {
        /// The frame deleted.
        frame: u16,
    },
    /// Moves a frame in play order; tags keep their positions.
    Move {
        /// The frame moved.
        frame: u16,
        /// Its new position.
        position: u16,
    },
    /// Changes how long a frame shows.
    SetDuration {
        /// The frame changed.
        frame: u16,
        /// How long it shows.
        duration_ms: u32,
    },
}

impl FrameEdit {
    /// The `core` operation of this edit, its positions read on `animation` as it is now.
    fn to_operation(&self, animation: &Animation) -> Result<Operation, EditError> {
        let operation = match *self {
            Self::Add {
                position,
                duration_ms,
            } => Operation::AddFrame {
                position: position.map_or_else(|| frame_count(animation), u32::from),
                duration_ms: duration_ms.unwrap_or(u32::from(DEFAULT_FRAME_DURATION_MS)),
            },
            Self::Duplicate { frame } => Operation::DuplicateFrame {
                frame: frame_at(animation, frame)?,
            },
            Self::Delete { frame } => Operation::DeleteFrame {
                frame: frame_at(animation, frame)?,
            },
            Self::Move { frame, position } => Operation::MoveFrame {
                frame: frame_at(animation, frame)?,
                position: u32::from(position),
            },
            Self::SetDuration { frame, duration_ms } => Operation::SetFrameDuration {
                frame: frame_at(animation, frame)?,
                duration_ms,
            },
        };
        Ok(operation)
    }
}

/// Applies `edits` in order, each on the frames the previous ones left.
pub(super) fn edit_frames(
    animation: &mut Animation,
    edits: &[FrameEdit],
) -> Result<(), EditingError> {
    for frame_edit in edits {
        let operation = frame_edit.to_operation(animation)?;
        let _undo = edit::apply(animation, &operation)?;
    }
    Ok(())
}

fn frame_count(animation: &Animation) -> u32 {
    u32::try_from(animation.frames().len()).unwrap_or(u32::MAX)
}
