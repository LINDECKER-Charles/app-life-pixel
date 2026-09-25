use alloc::vec::Vec;

use super::{AnimationData, EncodeError};

/// Encodes `animation` as payload v1. The same animation always gives the same bytes.
///
/// # Errors
///
/// [`EncodeError::BeyondBound`] when a size or count passes a bound,
/// [`EncodeError::Inconsistent`] when a frame or a tag does not fit the animation.
#[expect(unused_variables, reason = "P1 writes the bodies")]
pub fn encode(animation: &AnimationData) -> Result<Vec<u8>, EncodeError> {
    todo!("P1: encode payload v1")
}
