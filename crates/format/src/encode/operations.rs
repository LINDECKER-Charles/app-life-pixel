//! A frame's indices as operations, following the encoding rules of payload v1.

use alloc::vec::Vec;

use crate::layout::operation::{LITERAL, RUN, SKIP};
use crate::layout::{EXTENDED_COUNT_BASE, EXTENDED_COUNT_MARKER, OPERATION_SHIFT};
use crate::varint;

/// The shortest run of equal indices written as a RUN.
const MIN_RUN: usize = 3;

/// The shortest run of unchanged pixels written as a SKIP.
const MIN_SKIP: usize = 2;

/// Writes `indices` as a key frame: runs of 3 or more equal indices as RUN, the rest as
/// LITERAL.
pub(super) fn write_key(indices: &[u8], out: &mut Vec<u8>) {
    let mut literal_start = 0;
    let mut position = 0;
    while let Some((&index, rest)) = indices.get(position..).and_then(<[u8]>::split_first) {
        let run = 1 + rest.iter().take_while(|&&next| next == index).count();
        if run >= MIN_RUN {
            write_literal(
                indices.get(literal_start..position).unwrap_or_default(),
                out,
            );
            write_operation(RUN, run, out);
            out.push(index);
            literal_start = position + run;
        }
        position += run;
    }
    write_literal(indices.get(literal_start..).unwrap_or_default(), out);
}

/// Writes `indices` as a delta over `previous`: runs of 2 or more unchanged pixels as SKIP, the
/// rest as in a key frame.
pub(super) fn write_delta(previous: &[u8], indices: &[u8], out: &mut Vec<u8>) {
    let mut changed_start = 0;
    let mut position = 0;
    while position < indices.len() {
        let unchanged = unchanged_len(previous, indices, position);
        if unchanged >= MIN_SKIP {
            write_key(
                indices.get(changed_start..position).unwrap_or_default(),
                out,
            );
            write_operation(SKIP, unchanged, out);
            changed_start = position + unchanged;
        }
        position += unchanged.max(1);
    }
    write_key(indices.get(changed_start..).unwrap_or_default(), out);
}

/// The number of pixels from `position` on whose index `indices` keeps from `previous`.
fn unchanged_len(previous: &[u8], indices: &[u8], position: usize) -> usize {
    let previous = previous.get(position..).unwrap_or_default();
    let indices = indices.get(position..).unwrap_or_default();
    previous
        .iter()
        .zip(indices)
        .take_while(|(before, after)| before == after)
        .count()
}

fn write_literal(indices: &[u8], out: &mut Vec<u8>) {
    if indices.is_empty() {
        return;
    }
    write_operation(LITERAL, indices.len(), out);
    out.extend_from_slice(indices);
}

/// Writes an operation byte for `count` pixels, `count ≥ 1`, then its varint if it needs one.
fn write_operation(operation: u8, count: usize, out: &mut Vec<u8>) {
    let code = operation << OPERATION_SHIFT;
    match u8::try_from(count.saturating_sub(1)) {
        Ok(n) if n < EXTENDED_COUNT_MARKER => out.push(code | n),
        _ => {
            out.push(code | EXTENDED_COUNT_MARKER);
            varint::write(count.saturating_sub(EXTENDED_COUNT_BASE), out);
        }
    }
}
