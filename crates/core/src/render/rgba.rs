use super::composite::{Replacement, composite, composite_with};
use crate::model::{Animation, FrameId, Palette};

/// The pixels of `frame`: its [`composite`] through the palette, 4 bytes per pixel — red, green,
/// blue, alpha —, alpha not premultiplied.
#[must_use]
pub fn rgba(animation: &Animation, frame: FrameId) -> Vec<u8> {
    to_rgba(animation.palette(), &composite(animation, frame))
}

/// [`rgba`] of [`composite_with`]: `frame` with `replacement`'s cel, for previews.
#[must_use]
pub fn rgba_with(animation: &Animation, frame: FrameId, replacement: Replacement) -> Vec<u8> {
    to_rgba(
        animation.palette(),
        &composite_with(animation, frame, replacement),
    )
}

/// An index outside the palette, which a valid animation never holds, shows transparent.
fn to_rgba(palette: &Palette, indices: &[u8]) -> Vec<u8> {
    indices
        .iter()
        .flat_map(|&index| palette.get(index).unwrap_or_default().to_bytes())
        .collect()
}
