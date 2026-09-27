use super::operations::{Canvas, Operation, for_each_operation};
use crate::{DecodeError, Frame};

/// Applies a frame's operations to `indices`, `width × height` palette indices, row by row from
/// the top-left pixel. For a delta frame, `indices` holds the previous frame's indices.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when the operations break a rule of payload v1 — even though
/// [`Payload::parse`](crate::Payload::parse) already checked them, this never trusts its input.
/// The operations before the broken one are applied.
pub fn apply_frame(
    frame: &Frame<'_>,
    indices: &mut [u8],
    palette_len: u16,
) -> Result<(), DecodeError> {
    let canvas = Canvas {
        pixel_count: indices.len(),
        palette_len,
    };
    for_each_operation(frame, canvas, |pixels, operation| {
        let target = indices.get_mut(pixels).ok_or(DecodeError::Malformed)?;
        match operation {
            Operation::Skip => {}
            Operation::Run(index) => target.fill(index),
            Operation::Literal(source) => {
                target
                    .iter_mut()
                    .zip(source)
                    .for_each(|(pixel, &index)| *pixel = index);
            }
        }
        Ok(())
    })
}
