//! The frames section: each frame as a key or a delta frame, whichever the rules pick.

use alloc::vec::Vec;

use super::operations::{write_delta, write_key};
use super::sections::write_u16;
use super::{AnimationData, EncodeError};
use crate::FrameKind;
use crate::bounds::MAX_PAYLOAD_BYTES;

/// Writes every frame. Frame 0 and the first frame of every tag are key frames; any other frame
/// is a delta frame when its delta encoding is smaller than its key encoding.
pub(super) fn write_frames(
    animation: &AnimationData,
    out: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    let mut previous: Option<&[u8]> = None;
    for (position, frame) in animation.frames.iter().enumerate() {
        let starts_a_tag = animation
            .tags
            .iter()
            .any(|tag| usize::from(tag.first) == position);
        let delta_base = previous.filter(|_| !starts_a_tag);
        let (kind, data) = encode_frame(&frame.indices, delta_base);
        write_u16(frame.duration_ms, out);
        out.push(kind.to_byte());
        let data_len = u32::try_from(data.len()).map_err(|_| EncodeError::BeyondBound)?;
        out.extend_from_slice(&data_len.to_le_bytes());
        out.extend_from_slice(&data);
        check_payload_len(out)?;
        previous = Some(&frame.indices);
    }
    Ok(())
}

/// The frame's kind and data: its key encoding, or its delta encoding over `previous` when that
/// one is smaller.
fn encode_frame(indices: &[u8], previous: Option<&[u8]>) -> (FrameKind, Vec<u8>) {
    let mut key = Vec::new();
    write_key(indices, &mut key);
    let Some(previous) = previous else {
        return (FrameKind::Key, key);
    };
    let mut delta = Vec::new();
    write_delta(previous, indices, &mut delta);
    if delta.len() < key.len() {
        return (FrameKind::Delta, delta);
    }
    (FrameKind::Key, key)
}

/// Stops as soon as the payload passes [`MAX_PAYLOAD_BYTES`], before writing any more of it.
fn check_payload_len(payload: &[u8]) -> Result<(), EncodeError> {
    let is_beyond_bound = u32::try_from(payload.len()).map_or(true, |len| len > MAX_PAYLOAD_BYTES);
    if is_beyond_bound {
        return Err(EncodeError::BeyondBound);
    }
    Ok(())
}
