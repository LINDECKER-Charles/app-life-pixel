//! Why a document, or a text grid, is refused: a stable code and its parameters, never a sentence
//! for the user — the interface translates `errors.<code>`.

use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::limits::{
    CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, MAX_CEL_PIXELS, MAX_DOCUMENT_BYTES, MAX_FRAME_DURATION_MS,
    MAX_FRAMES, MAX_LAYERS, MAX_PALETTE_ENTRIES, MAX_TAGS, MIN_FRAME_DURATION_MS, NAME_MAX_CHARS,
};

/// Every code of this crate, each with its key `errors.<code>` in every catalogue.
pub const CODES: &[&str] = &[
    "document.canvas_size",
    "document.cel",
    "document.frame_count",
    "document.frame_duration",
    "document.layer_count",
    "document.malformed",
    "document.name",
    "document.palette",
    "document.pixel_budget",
    "document.reference",
    "document.tag",
    "document.tag_count",
    "document.too_large",
    "document.unsupported_version",
    "grid.character",
    "grid.index",
    "grid.size",
];

/// A broken rule of the document model, or of the text grid. Rows and columns count from 0.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum DocumentError {
    /// Not JSON, a missing or unknown field, a wrong type.
    #[error("malformed document")]
    Malformed,
    /// A document version this build does not know.
    #[error("unsupported document version {version}")]
    UnsupportedVersion {
        /// The version the document declares.
        version: u64,
    },
    /// More than [`MAX_DOCUMENT_BYTES`].
    #[error("document larger than {MAX_DOCUMENT_BYTES} bytes")]
    TooLarge,
    /// A side out of [`CANVAS_MIN_SIDE`] to [`CANVAS_MAX_SIDE`].
    #[error("canvas side out of {CANVAS_MIN_SIDE} to {CANVAS_MAX_SIDE}")]
    CanvasSize,
    /// No frame, or more than [`MAX_FRAMES`].
    #[error("frame count out of 1 to {MAX_FRAMES}")]
    FrameCount,
    /// No layer, or more than [`MAX_LAYERS`].
    #[error("layer count out of 1 to {MAX_LAYERS}")]
    LayerCount,
    /// More than [`MAX_TAGS`].
    #[error("more than {MAX_TAGS} tags")]
    TagCount,
    /// Non-empty cels beyond [`MAX_CEL_PIXELS`] pixels together.
    #[error("cels beyond {MAX_CEL_PIXELS} pixels")]
    PixelBudget,
    /// No entry, more than [`MAX_PALETTE_ENTRIES`], entry 0 not transparent, or a colour not
    /// `#rrggbbaa`.
    #[error("invalid palette")]
    Palette,
    /// A duration out of [`MIN_FRAME_DURATION_MS`] to [`MAX_FRAME_DURATION_MS`].
    #[error("frame duration out of {MIN_FRAME_DURATION_MS} to {MAX_FRAME_DURATION_MS} ms")]
    FrameDuration,
    /// An invalid title, project name or layer name.
    #[error("invalid name")]
    Name,
    /// An invalid, duplicate or out-of-range tag.
    #[error("invalid tag {name:?}")]
    Tag {
        /// The tag's name, as given.
        name: String,
    },
    /// A duplicate id, a cel on a missing layer or frame, `nextId` not above every id.
    #[error("broken reference")]
    Reference,
    /// A cel of the wrong size, undecodable, or with an index at or above the palette size.
    #[error("invalid cel")]
    Cel,
    /// A grid without `height` rows of `width` pixels.
    #[error("grid is not {width} × {height}")]
    GridSize {
        /// The canvas width, in pixels.
        width: u16,
        /// The canvas height, in pixels.
        height: u16,
    },
    /// A character outside the grid's alphabet.
    #[error("invalid grid character at row {row}, column {column}")]
    GridCharacter {
        /// The row of the pixel.
        row: usize,
        /// The column of the pixel.
        column: usize,
    },
    /// An index at or above the palette size.
    #[error("grid index {index} at row {row}, column {column} is not in the palette")]
    GridIndex {
        /// The row of the pixel.
        row: usize,
        /// The column of the pixel.
        column: usize,
        /// The palette index found.
        index: u8,
    },
}

impl DocumentError {
    /// The stable code of this error, one of [`CODES`].
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Malformed => "document.malformed",
            Self::UnsupportedVersion { .. } => "document.unsupported_version",
            Self::TooLarge => "document.too_large",
            Self::CanvasSize => "document.canvas_size",
            Self::FrameCount => "document.frame_count",
            Self::LayerCount => "document.layer_count",
            Self::TagCount => "document.tag_count",
            Self::PixelBudget => "document.pixel_budget",
            Self::Palette => "document.palette",
            Self::FrameDuration => "document.frame_duration",
            Self::Name => "document.name",
            Self::Tag { .. } => "document.tag",
            Self::Reference => "document.reference",
            Self::Cel => "document.cel",
            Self::GridSize { .. } => "grid.size",
            Self::GridCharacter { .. } => "grid.character",
            Self::GridIndex { .. } => "grid.index",
        }
    }

    /// The parameters of this error, named in camelCase as the catalogues' messages use them.
    #[must_use]
    pub fn params(&self) -> Map<String, Value> {
        let params = match self {
            Self::Malformed | Self::Reference | Self::Cel => json!({}),
            Self::UnsupportedVersion { version } => json!({ "version": version }),
            Self::TooLarge => json!({ "maxBytes": MAX_DOCUMENT_BYTES }),
            Self::CanvasSize => json!({ "min": CANVAS_MIN_SIDE, "max": CANVAS_MAX_SIDE }),
            Self::FrameCount => json!({ "max": MAX_FRAMES }),
            Self::LayerCount => json!({ "max": MAX_LAYERS }),
            Self::TagCount => json!({ "max": MAX_TAGS }),
            Self::PixelBudget => json!({ "max": MAX_CEL_PIXELS }),
            Self::Palette => json!({ "max": MAX_PALETTE_ENTRIES }),
            Self::FrameDuration => {
                json!({ "min": MIN_FRAME_DURATION_MS, "max": MAX_FRAME_DURATION_MS })
            }
            Self::Name => json!({ "max": NAME_MAX_CHARS }),
            Self::Tag { name } => json!({ "name": name }),
            Self::GridSize { width, height } => json!({ "width": width, "height": height }),
            Self::GridCharacter { row, column } => json!({ "row": row, "column": column }),
            Self::GridIndex { row, column, index } => {
                json!({ "row": row, "column": column, "index": index })
            }
        };
        match params {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }
}
