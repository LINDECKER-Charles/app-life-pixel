use super::frames::{Frames, read_frames};
use super::header::{Header, read_header};
use super::operations::Canvas;
use super::reader::Reader;
use super::sections::{Palette, Tags, read_palette, read_tag, read_tags, read_title};
use crate::bounds::MAX_PAYLOAD_BYTES;
use crate::layout::PALETTE_ENTRY_BYTES;
use crate::{DecodeError, Frame, Rgba, Tag};

/// A payload whose every byte has been checked; it borrows the bytes it was parsed from.
#[derive(Clone, Copy, Debug)]
pub struct Payload<'a> {
    header: Header,
    palette: Palette<'a>,
    title: &'a str,
    tags: Tags<'a>,
    frames: &'a [u8],
}

impl<'a> Payload<'a> {
    /// Parses and checks the whole payload, every frame's operations included, so that a parsed
    /// payload never fails later.
    ///
    /// # Errors
    ///
    /// The first rule of payload v1 that `bytes` breaks, as a [`DecodeError`].
    pub fn parse(bytes: &'a [u8]) -> Result<Payload<'a>, DecodeError> {
        let is_beyond_bound =
            u32::try_from(bytes.len()).map_or(true, |len| len > MAX_PAYLOAD_BYTES);
        if is_beyond_bound {
            return Err(DecodeError::BeyondBound);
        }
        let mut reader = Reader::new(bytes);
        let header = read_header(&mut reader)?;
        let palette = read_palette(&mut reader)?;
        let title = read_title(&mut reader)?;
        let tags = read_tags(&mut reader, header.frame_count)?;
        let canvas = Canvas {
            pixel_count: header.pixel_count(),
            palette_len: palette.len,
        };
        let frames = read_frames(&mut reader, header.frame_count, canvas)?;
        Ok(Payload {
            header,
            palette,
            title,
            tags,
            frames,
        })
    }

    /// The canvas width, in pixels.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.header.width
    }

    /// The canvas height, in pixels.
    #[must_use]
    pub fn height(&self) -> u16 {
        self.header.height
    }

    /// The number of palette entries, 1 to 256.
    #[must_use]
    pub fn palette_len(&self) -> u16 {
        self.palette.len
    }

    /// The palette entry at `index`, or `None` at or beyond the palette count.
    #[must_use]
    pub fn palette_entry(&self, index: u8) -> Option<Rgba> {
        let start = usize::from(index) * PALETTE_ENTRY_BYTES;
        let entry = self
            .palette
            .entries
            .get(start..start + PALETTE_ENTRY_BYTES)?;
        let &[r, g, b, a] = entry else {
            return None;
        };
        Some(Rgba { r, g, b, a })
    }

    /// The title, possibly empty.
    #[must_use]
    pub fn title(&self) -> &'a str {
        self.title
    }

    /// The number of tags.
    #[must_use]
    pub fn tag_count(&self) -> u16 {
        self.tags.count
    }

    /// The tag at `index`, or `None` at or beyond the tag count.
    #[must_use]
    pub fn tag(&self, index: u16) -> Option<Tag<'a>> {
        if index >= self.tags.count {
            return None;
        }
        let mut reader = Reader::new(self.tags.bytes);
        let frame_count = self.header.frame_count;
        for _ in 0..index {
            read_tag(&mut reader, frame_count).ok()?;
        }
        read_tag(&mut reader, frame_count).ok()
    }

    /// The number of frames, at least 1.
    #[must_use]
    pub fn frame_count(&self) -> u16 {
        self.header.frame_count
    }

    /// The frames, in order.
    pub fn frames(&self) -> impl Iterator<Item = Frame<'a>> + 'a {
        Frames::new(self.frames, self.header.frame_count)
    }
}
