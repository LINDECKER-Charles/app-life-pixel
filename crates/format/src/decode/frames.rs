use super::operations::{Canvas, for_each_operation};
use super::reader::Reader;
use crate::{DecodeError, Frame, FrameKind};

/// Reads one frame: a duration of at least 1 ms, a kind of 0 or 1, then its data.
pub(super) fn read_frame<'a>(reader: &mut Reader<'a>) -> Result<Frame<'a>, DecodeError> {
    let duration_ms = reader.u16()?;
    if duration_ms == 0 {
        return Err(DecodeError::Malformed);
    }
    let kind = FrameKind::from_byte(reader.u8()?).ok_or(DecodeError::Malformed)?;
    let data_len = usize::try_from(reader.u32()?).map_err(|_| DecodeError::Malformed)?;
    let data = reader.bytes(data_len)?;
    Ok(Frame {
        duration_ms,
        kind,
        data,
    })
}

/// Reads and checks `frame_count` frames, every operation included, up to the end of the
/// payload: frame 0 is a key frame, and no byte follows the last frame.
pub(super) fn read_frames<'a>(
    reader: &mut Reader<'a>,
    frame_count: u16,
    canvas: Canvas,
) -> Result<&'a [u8], DecodeError> {
    let frames = reader.rest();
    for position in 0..frame_count {
        let frame = read_frame(reader)?;
        if position == 0 && frame.kind == FrameKind::Delta {
            return Err(DecodeError::Malformed);
        }
        for_each_operation(&frame, canvas, |_, _| Ok(()))?;
    }
    if !reader.is_empty() {
        return Err(DecodeError::Malformed);
    }
    Ok(frames)
}

/// The frames of a checked payload, in order.
#[derive(Clone, Debug)]
pub(super) struct Frames<'a> {
    reader: Reader<'a>,
    remaining: u16,
}

impl<'a> Frames<'a> {
    pub(super) fn new(bytes: &'a [u8], frame_count: u16) -> Self {
        Self {
            reader: Reader::new(bytes),
            remaining: frame_count,
        }
    }
}

impl<'a> Iterator for Frames<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Frame<'a>> {
        self.remaining = self.remaining.checked_sub(1)?;
        let frame = read_frame(&mut self.reader).ok();
        if frame.is_none() {
            self.remaining = 0;
        }
        frame
    }
}
