//! A frame as RGBA, through `core::render` — with the cel an operation in progress would give, for
//! the canvas's preview.

use life_pixel_core::edit::{self, EditError, Operation};
use life_pixel_core::{Animation, FrameId, render};
use serde::{Deserialize, Serialize};

use crate::errors::EngineError;

/// The interface's `RenderRequest`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderRequest {
    /// The frame rendered.
    pub frame: FrameId,
    /// A pixel operation in progress, drawn without being applied.
    pub preview: Option<Operation>,
}

/// The interface's `RenderedFrame`: 4 bytes per pixel, row by row — red, green, blue, alpha.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RenderedFrame {
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// The pixels, a `Uint8Array` in JavaScript.
    #[serde(with = "serde_bytes")]
    pub pixels: Vec<u8>,
}

/// The pixels of `request.frame`; with a preview on that frame, its layer shows the cel the
/// operation would give. The animation never changes.
///
/// # Errors
///
/// `edit.frame_not_found` for a frame the animation lacks, or the error the preview's operation
/// would fail with.
pub fn render_frame(
    animation: &Animation,
    request: &RenderRequest,
) -> Result<RenderedFrame, EngineError> {
    if animation.frame(request.frame).is_none() {
        return Err(EditError::FrameNotFound.into());
    }
    let preview = match &request.preview {
        Some(operation) => edit::preview(animation, operation)?,
        None => None,
    };
    let pixels = match preview {
        Some((layer, frame, cel)) if frame == request.frame => {
            render::rgba_with(animation, frame, (layer, &cel))
        }
        _ => render::rgba(animation, request.frame),
    };
    Ok(RenderedFrame {
        width: animation.width(),
        height: animation.height(),
        pixels,
    })
}
