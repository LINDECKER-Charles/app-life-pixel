//! The header and the sections before the frames: palette, title and tags.

use alloc::vec::Vec;

use super::{AnimationData, EncodeError, TagData};
use crate::layout::RESERVED;
use crate::{ABI_VERSION, FORMAT_VERSION, MAGIC, Rgba};

/// Writes the 16 bytes of the header.
pub(super) fn write_header(
    animation: &AnimationData,
    out: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    out.extend_from_slice(&MAGIC);
    write_u16(FORMAT_VERSION, out);
    write_u16(ABI_VERSION, out);
    write_u16(animation.width, out);
    write_u16(animation.height, out);
    write_u16(len_u16(animation.frames.len())?, out);
    write_u16(RESERVED, out);
    Ok(())
}

/// Writes the palette's entry count, then its entries.
pub(super) fn write_palette(palette: &[Rgba], out: &mut Vec<u8>) -> Result<(), EncodeError> {
    write_u16(len_u16(palette.len())?, out);
    for entry in palette {
        out.extend_from_slice(&[entry.r, entry.g, entry.b, entry.a]);
    }
    Ok(())
}

/// Writes the title's length, then its bytes.
pub(super) fn write_title(title: &str, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    write_u16(len_u16(title.len())?, out);
    out.extend_from_slice(title.as_bytes());
    Ok(())
}

/// Writes the tag count, then each tag.
pub(super) fn write_tags(tags: &[TagData], out: &mut Vec<u8>) -> Result<(), EncodeError> {
    write_u16(len_u16(tags.len())?, out);
    for tag in tags {
        write_u16(tag.first, out);
        write_u16(tag.last, out);
        out.push(tag.loop_mode.to_byte());
        out.push(u8::try_from(tag.name.len()).map_err(|_| EncodeError::BeyondBound)?);
        out.extend_from_slice(tag.name.as_bytes());
    }
    Ok(())
}

pub(super) fn write_u16(value: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A length that must fit a `u16` field.
fn len_u16(len: usize) -> Result<u16, EncodeError> {
    u16::try_from(len).map_err(|_| EncodeError::BeyondBound)
}
