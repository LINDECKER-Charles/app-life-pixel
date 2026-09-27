//! The rules an animation must follow to be encoded: the rules of payload v1, which the decoder
//! checks, stated over the animation. What the decoder calls beyond a bound is
//! [`EncodeError::BeyondBound`]; the rest is [`EncodeError::Inconsistent`].

use super::{AnimationData, EncodeError, FrameData, TagData};
use crate::Rgba;
use crate::bounds::{MAX_FRAMES, MAX_SIDE, MAX_TAG_NAME_BYTES, MAX_TAGS, MAX_TITLE_BYTES};
use crate::layout::{MAX_PALETTE_ENTRIES, TRANSPARENT};

/// Checks every rule of payload v1 that `animation` could break.
pub(super) fn check_animation(animation: &AnimationData) -> Result<(), EncodeError> {
    check_count(animation.width.into(), MAX_SIDE.into())?;
    check_count(animation.height.into(), MAX_SIDE.into())?;
    check_count(animation.frames.len(), MAX_FRAMES.into())?;
    check_palette(&animation.palette)?;
    check_bound(animation.title.len(), MAX_TITLE_BYTES.into())?;
    check_bound(animation.tags.len(), MAX_TAGS.into())?;
    let frame_count = animation.frames.len();
    for tag in &animation.tags {
        check_tag(tag, frame_count)?;
    }
    let pixel_count = usize::from(animation.width) * usize::from(animation.height);
    for frame in &animation.frames {
        check_frame(frame, pixel_count, animation.palette.len())?;
    }
    Ok(())
}

/// 1 to 256 entries, entry 0 fully transparent.
fn check_palette(palette: &[Rgba]) -> Result<(), EncodeError> {
    check_count(palette.len(), MAX_PALETTE_ENTRIES.into())
        .map_err(|_| EncodeError::Inconsistent)?;
    let is_first_transparent = palette
        .first()
        .is_some_and(|entry| [entry.r, entry.g, entry.b, entry.a] == TRANSPARENT);
    consistent_if(is_first_transparent)
}

/// A name of 1 to [`MAX_TAG_NAME_BYTES`] bytes, and `first ≤ last < frame_count`.
fn check_tag(tag: &TagData, frame_count: usize) -> Result<(), EncodeError> {
    check_count(tag.name.len(), MAX_TAG_NAME_BYTES.into())?;
    consistent_if(tag.first <= tag.last && usize::from(tag.last) < frame_count)
}

/// A duration of at least 1 ms, and `pixel_count` indices, each below `palette_len`.
fn check_frame(
    frame: &FrameData,
    pixel_count: usize,
    palette_len: usize,
) -> Result<(), EncodeError> {
    let is_in_palette = |index: &u8| usize::from(*index) < palette_len;
    consistent_if(
        frame.duration_ms > 0
            && frame.indices.len() == pixel_count
            && frame.indices.iter().all(is_in_palette),
    )
}

/// A count from 1 to `max`: 0 is inconsistent, above `max` beyond a bound.
fn check_count(count: usize, max: usize) -> Result<(), EncodeError> {
    consistent_if(count > 0)?;
    check_bound(count, max)
}

fn check_bound(value: usize, max: usize) -> Result<(), EncodeError> {
    if value > max {
        return Err(EncodeError::BeyondBound);
    }
    Ok(())
}

fn consistent_if(is_consistent: bool) -> Result<(), EncodeError> {
    if !is_consistent {
        return Err(EncodeError::Inconsistent);
    }
    Ok(())
}
