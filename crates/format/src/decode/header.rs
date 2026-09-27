use super::reader::Reader;
use crate::bounds::{MAX_FRAMES, MAX_SIDE};
use crate::layout::RESERVED;
use crate::{ABI_VERSION, DecodeError, FORMAT_VERSION, MAGIC};

/// The header fields a parsed payload keeps: the canvas and the number of frames.
#[derive(Clone, Copy, Debug)]
pub(super) struct Header {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) frame_count: u16,
}

impl Header {
    /// The number of pixels of a frame, `width × height`.
    pub(super) fn pixel_count(&self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }
}

/// Reads and checks the 16 bytes of the header, field by field.
pub(super) fn read_header(reader: &mut Reader<'_>) -> Result<Header, DecodeError> {
    if reader.array()? != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    if reader.u16()? != FORMAT_VERSION {
        return Err(DecodeError::UnknownFormatVersion);
    }
    if reader.u16()? != ABI_VERSION {
        return Err(DecodeError::UnknownAbiVersion);
    }
    let width = read_count(reader, MAX_SIDE)?;
    let height = read_count(reader, MAX_SIDE)?;
    let frame_count = read_count(reader, MAX_FRAMES)?;
    if reader.u16()? != RESERVED {
        return Err(DecodeError::Malformed);
    }
    Ok(Header {
        width,
        height,
        frame_count,
    })
}

/// A `u16` from 1 to `max`: 0 is malformed, above `max` beyond a bound.
fn read_count(reader: &mut Reader<'_>, max: u16) -> Result<u16, DecodeError> {
    let value = reader.u16()?;
    if value == 0 {
        return Err(DecodeError::Malformed);
    }
    check_bound(value, max)
}

/// `value` itself, or [`DecodeError::BeyondBound`] above `max`.
pub(super) fn check_bound<T: PartialOrd>(value: T, max: T) -> Result<T, DecodeError> {
    if value > max {
        return Err(DecodeError::BeyondBound);
    }
    Ok(value)
}
