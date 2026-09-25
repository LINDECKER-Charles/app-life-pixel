use crate::{DecodeError, Frame};

/// Applies a frame's operations to `indices`, `width × height` palette indices, row by row from
/// the top-left pixel. For a delta frame, `indices` holds the previous frame's indices.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the operations break a rule of payload v1 — even though
/// [`Payload::parse`](crate::Payload::parse) already checked them, this never trusts its input.
#[expect(unused_variables, reason = "P1 writes the bodies")]
pub fn apply_frame(
    frame: &Frame<'_>,
    indices: &mut [u8],
    palette_len: u16,
) -> Result<(), DecodeError> {
    todo!("P1: apply a frame's operations")
}
