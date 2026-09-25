//! The sections between the header and the frames: palette, title and tags.

use super::header::check_bound;
use super::reader::Reader;
use crate::bounds::{MAX_TAG_NAME_BYTES, MAX_TAGS, MAX_TITLE_BYTES};
use crate::layout::{MAX_PALETTE_ENTRIES, PALETTE_ENTRY_BYTES, TRANSPARENT};
use crate::{DecodeError, LoopMode, Tag};

/// A checked palette: its entry count and its entries' bytes.
#[derive(Clone, Copy, Debug)]
pub(super) struct Palette<'a> {
    pub(super) len: u16,
    pub(super) entries: &'a [u8],
}

/// A checked tags section: its tag count and its tags' bytes.
#[derive(Clone, Copy, Debug)]
pub(super) struct Tags<'a> {
    pub(super) count: u16,
    pub(super) bytes: &'a [u8],
}

/// Reads the palette: 1 to 256 entries, entry 0 fully transparent.
pub(super) fn read_palette<'a>(reader: &mut Reader<'a>) -> Result<Palette<'a>, DecodeError> {
    let len = reader.u16()?;
    if len == 0 || len > MAX_PALETTE_ENTRIES {
        return Err(DecodeError::Malformed);
    }
    let entries = reader.bytes(usize::from(len) * PALETTE_ENTRY_BYTES)?;
    if entries.get(..PALETTE_ENTRY_BYTES) != Some(&TRANSPARENT[..]) {
        return Err(DecodeError::Malformed);
    }
    Ok(Palette { len, entries })
}

/// Reads the title: up to [`MAX_TITLE_BYTES`] bytes of UTF-8.
pub(super) fn read_title<'a>(reader: &mut Reader<'a>) -> Result<&'a str, DecodeError> {
    let len = check_bound(reader.u16()?, MAX_TITLE_BYTES)?;
    reader.str(usize::from(len))
}

/// Reads the tags section, checking every tag against `frame_count`.
pub(super) fn read_tags<'a>(
    reader: &mut Reader<'a>,
    frame_count: u16,
) -> Result<Tags<'a>, DecodeError> {
    let count = check_bound(reader.u16()?, MAX_TAGS)?;
    let start = *reader;
    for _ in 0..count {
        read_tag(reader, frame_count)?;
    }
    let bytes = reader.read_since(start);
    Ok(Tags { count, bytes })
}

/// Reads one tag: `first ≤ last < frame_count`, a loop mode of 0 or 1, and a name of 1 to
/// [`MAX_TAG_NAME_BYTES`] bytes of UTF-8.
pub(super) fn read_tag<'a>(
    reader: &mut Reader<'a>,
    frame_count: u16,
) -> Result<Tag<'a>, DecodeError> {
    let first = reader.u16()?;
    let last = reader.u16()?;
    if first > last || last >= frame_count {
        return Err(DecodeError::Malformed);
    }
    let loop_mode = LoopMode::from_byte(reader.u8()?).ok_or(DecodeError::Malformed)?;
    let name_len = reader.u8()?;
    if name_len == 0 {
        return Err(DecodeError::Malformed);
    }
    let name_len = check_bound(name_len, MAX_TAG_NAME_BYTES)?;
    let name = reader.str(usize::from(name_len))?;
    Ok(Tag {
        name,
        first,
        last,
        loop_mode,
    })
}
