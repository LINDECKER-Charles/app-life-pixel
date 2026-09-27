use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::error::DocumentError;
use crate::limits::{IMPORT_MAX_BYTES, IMPORT_MAX_SIDE, MAX_PALETTE_ENTRIES, STROKE_MAX_POINTS};

/// Why an operation is refused: a stable code and its parameters, never a sentence for the user.
/// A refused operation changes nothing.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum EditError {
    /// The animation would break a rule of the model: the model's own code.
    #[error(transparent)]
    Document(#[from] DocumentError),
    /// No layer of the animation has this id.
    #[error("layer not found")]
    LayerNotFound,
    /// No frame of the animation has this id.
    #[error("frame not found")]
    FrameNotFound,
    /// No tag of the animation has this name.
    #[error("tag {name:?} not found")]
    TagNotFound {
        /// The name looked for.
        name: String,
    },
    /// A point that must lie on the canvas lies outside it.
    #[error("point outside the canvas")]
    OutOfCanvas,
    /// Deleting the only layer.
    #[error("the only layer cannot be deleted")]
    LastLayer,
    /// Deleting the only frame.
    #[error("the only frame cannot be deleted")]
    LastFrame,
    /// Adding a palette entry beyond [`MAX_PALETTE_ENTRIES`].
    #[error("palette full at {MAX_PALETTE_ENTRIES} entries")]
    PaletteFull,
    /// A layer or frame position beyond the list it goes into.
    #[error("position above {max}")]
    PositionOutOfRange {
        /// The highest position allowed.
        max: usize,
    },
    /// A stroke of more than [`STROKE_MAX_POINTS`] points.
    #[error("stroke longer than {STROKE_MAX_POINTS} points")]
    StrokeTooLong,
    /// An image file above [`IMPORT_MAX_BYTES`], or a side above [`IMPORT_MAX_SIDE`].
    #[error("image larger than {IMPORT_MAX_SIDE} pixels a side or {IMPORT_MAX_BYTES} bytes")]
    ImageTooLarge,
    /// Not a PNG image, or a damaged one.
    #[error("malformed image")]
    ImageMalformed,
    /// A sprite sheet that is not a whole grid of cells of the given size.
    #[error("sprite sheet grid does not fit the image")]
    SheetGrid,
}

impl EditError {
    /// The stable code of this error, one of [`CODES`](crate::error::CODES).
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Document(error) => error.code(),
            Self::LayerNotFound => "edit.layer_not_found",
            Self::FrameNotFound => "edit.frame_not_found",
            Self::TagNotFound { .. } => "edit.tag_not_found",
            Self::OutOfCanvas => "edit.out_of_canvas",
            Self::LastLayer => "edit.last_layer",
            Self::LastFrame => "edit.last_frame",
            Self::PaletteFull => "edit.palette_full",
            Self::PositionOutOfRange { .. } => "edit.position_out_of_range",
            Self::StrokeTooLong => "edit.stroke_too_long",
            Self::ImageTooLarge => "import.image_too_large",
            Self::ImageMalformed => "import.image_malformed",
            Self::SheetGrid => "import.sheet_grid",
        }
    }

    /// The parameters of this error, named in camelCase as the catalogues' messages use them.
    #[must_use]
    pub fn params(&self) -> Map<String, Value> {
        let params = match self {
            Self::Document(error) => return error.params(),
            Self::TagNotFound { name } => json!({ "name": name }),
            Self::PaletteFull => json!({ "max": MAX_PALETTE_ENTRIES }),
            Self::PositionOutOfRange { max } => json!({ "max": max }),
            Self::StrokeTooLong => json!({ "max": STROKE_MAX_POINTS }),
            Self::ImageTooLarge => {
                json!({ "maxSide": IMPORT_MAX_SIDE, "maxBytes": IMPORT_MAX_BYTES })
            }
            Self::LayerNotFound
            | Self::FrameNotFound
            | Self::OutOfCanvas
            | Self::LastLayer
            | Self::LastFrame
            | Self::ImageMalformed
            | Self::SheetGrid => json!({}),
        };
        match params {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }
}
