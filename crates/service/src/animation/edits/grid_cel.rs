//! A text grid as a cel: the document's `grid` form of a cel — the one it keeps for what people
//! and agents write —, read back by `core`, which parses the grid and checks every rule.
//!
//! `core` has no operation that replaces a whole cel; drawing the grid run by run through its
//! pixel operations would copy the cel once per run. Reading the document costs one pass.

use life_pixel_core::edit::EditError;
use life_pixel_core::serialize::{grid, read_document, write_document};
use life_pixel_core::{Animation, DocumentError, FrameId, LayerId};
use serde_json::{Value, json};

use crate::animation::EditingError;
use crate::animation::addressing::Target;

/// The document's list of cels.
const CELS: &str = "cels";
/// A cel's layer id.
const LAYER: &str = "layer";
/// A cel's frame id.
const FRAME: &str = "frame";

/// Replaces the cel of `target` with the grid `rows`.
pub(super) fn write_grid(
    animation: &mut Animation,
    target: Target,
    rows: &[String],
) -> Result<(), EditingError> {
    let _checked = grid::parse(rows, animation.cel_shape())?;
    let (layer, frame) = target.resolve(animation)?;
    if animation.layer(layer).is_none() {
        return Err(EditError::LayerNotFound.into());
    }
    *animation = with_grid_cel(animation, (layer, frame), rows)?;
    Ok(())
}

/// `animation` with the cel of `(layer, frame)` written as `rows`.
fn with_grid_cel(
    animation: &Animation,
    (layer, frame): (LayerId, FrameId),
    rows: &[String],
) -> Result<Animation, DocumentError> {
    let text = write_document(animation)?;
    let mut document: Value = serde_json::from_str(&text).map_err(|_| DocumentError::Malformed)?;
    let cels = document.get_mut(CELS).and_then(Value::as_array_mut);
    let cels = cels.ok_or(DocumentError::Malformed)?;
    let key = (json!(layer.get()), json!(frame.get()));
    cels.retain(|cel| (&cel[LAYER], &cel[FRAME]) != (&key.0, &key.1));
    cels.push(json!({ LAYER: key.0, FRAME: key.1, "grid": rows }));
    let bytes = serde_json::to_vec(&document).map_err(|_| DocumentError::Malformed)?;
    read_document(&bytes)
}
