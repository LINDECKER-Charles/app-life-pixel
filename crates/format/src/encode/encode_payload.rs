use alloc::vec::Vec;

use super::frames::write_frames;
use super::sections::{write_header, write_palette, write_tags, write_title};
use super::validate::check_animation;
use super::{AnimationData, EncodeError};

/// Encodes `animation` as payload v1. The same animation always gives the same bytes.
///
/// # Errors
///
/// [`EncodeError::BeyondBound`] when a size or count passes a bound,
/// [`EncodeError::Inconsistent`] when a frame or a tag does not fit the animation.
pub fn encode(animation: &AnimationData) -> Result<Vec<u8>, EncodeError> {
    check_animation(animation)?;
    let mut payload = Vec::new();
    write_header(animation, &mut payload)?;
    write_palette(&animation.palette, &mut payload)?;
    write_title(&animation.title, &mut payload)?;
    write_tags(&animation.tags, &mut payload)?;
    write_frames(animation, &mut payload)?;
    Ok(payload)
}
