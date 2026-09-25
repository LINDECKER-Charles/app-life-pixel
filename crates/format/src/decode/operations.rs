use core::ops::Range;

use super::reader::Reader;
use crate::layout::operation::{LITERAL, RUN, SKIP};
use crate::layout::{COUNT_MASK, EXTENDED_COUNT_BASE, EXTENDED_COUNT_MARKER, OPERATION_SHIFT};
use crate::{DecodeError, Frame, FrameKind};

/// One operation of a frame's data.
#[derive(Clone, Copy, Debug)]
pub(super) enum Operation<'a> {
    /// The pixels keep the previous frame's indices.
    Skip,
    /// The pixels take this index.
    Run(u8),
    /// The pixels take these indices, in order.
    Literal(&'a [u8]),
}

/// What a frame's operations cover: its pixels and the palette they index.
#[derive(Clone, Copy, Debug)]
pub(super) struct Canvas {
    pub(super) pixel_count: usize,
    pub(super) palette_len: u16,
}

/// Reads and checks every operation of `frame`, passing each to `visit` with the pixels it
/// covers, which always lie within `canvas`. The frame's counts must add up to exactly
/// `canvas.pixel_count`, and its operations end exactly at the end of its data.
pub(super) fn for_each_operation<'a>(
    frame: &Frame<'a>,
    canvas: Canvas,
    mut visit: impl FnMut(Range<usize>, Operation<'a>) -> Result<(), DecodeError>,
) -> Result<(), DecodeError> {
    let mut reader = Reader::new(frame.data);
    let mut covered: usize = 0;
    while !reader.is_empty() {
        let (count, operation) = read_operation(&mut reader)?;
        check_operation(operation, frame.kind, canvas.palette_len)?;
        let end = covered
            .checked_add(count)
            .filter(|&end| end <= canvas.pixel_count)
            .ok_or(DecodeError::Malformed)?;
        visit(covered..end, operation)?;
        covered = end;
    }
    if covered != canvas.pixel_count {
        return Err(DecodeError::Malformed);
    }
    Ok(())
}

/// Reads one operation and the number of pixels it covers.
fn read_operation<'a>(reader: &mut Reader<'a>) -> Result<(usize, Operation<'a>), DecodeError> {
    let byte = reader.u8()?;
    let count = read_count(byte & COUNT_MASK, reader)?;
    let operation = match byte >> OPERATION_SHIFT {
        SKIP => Operation::Skip,
        RUN => Operation::Run(reader.u8()?),
        LITERAL => Operation::Literal(reader.bytes(count)?),
        _ => return Err(DecodeError::Malformed),
    };
    Ok((count, operation))
}

/// The count `n` stands for: `n + 1`, or 64 plus the varint that follows when `n` is 63.
fn read_count(n: u8, reader: &mut Reader<'_>) -> Result<usize, DecodeError> {
    if n != EXTENDED_COUNT_MARKER {
        return Ok(usize::from(n) + 1);
    }
    let extra = usize::try_from(reader.varint()?).map_err(|_| DecodeError::Malformed)?;
    extra
        .checked_add(EXTENDED_COUNT_BASE)
        .ok_or(DecodeError::Malformed)
}

/// Refuses a SKIP in a key frame and an index at or above `palette_len`.
fn check_operation(
    operation: Operation<'_>,
    kind: FrameKind,
    palette_len: u16,
) -> Result<(), DecodeError> {
    let is_in_palette = |index: &u8| u16::from(*index) < palette_len;
    let is_valid = match operation {
        Operation::Skip => kind == FrameKind::Delta,
        Operation::Run(index) => is_in_palette(&index),
        Operation::Literal(indices) => indices.iter().all(is_in_palette),
    };
    if !is_valid {
        return Err(DecodeError::Malformed);
    }
    Ok(())
}
