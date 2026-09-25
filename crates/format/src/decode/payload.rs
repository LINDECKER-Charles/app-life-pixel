use core::marker::PhantomData;

use crate::{DecodeError, Frame, Rgba, Tag};

/// A payload whose every byte has been checked; it borrows the bytes it was parsed from.
#[derive(Clone, Copy, Debug)]
pub struct Payload<'a> {
    // P1 replaces this marker with the bytes and the offsets of each section.
    bytes: PhantomData<&'a [u8]>,
}

#[expect(unused_variables, reason = "P1 writes the bodies")]
impl<'a> Payload<'a> {
    /// Parses and checks the whole payload, every frame's operations included, so that a parsed
    /// payload never fails later.
    ///
    /// # Errors
    ///
    /// The first rule of payload v1 that `bytes` breaks, as a [`DecodeError`].
    pub fn parse(bytes: &'a [u8]) -> Result<Payload<'a>, DecodeError> {
        todo!("P1: parse payload v1")
    }

    /// The canvas width, in pixels.
    #[must_use]
    pub fn width(&self) -> u16 {
        todo!("P1: read the width")
    }

    /// The canvas height, in pixels.
    #[must_use]
    pub fn height(&self) -> u16 {
        todo!("P1: read the height")
    }

    /// The number of palette entries, 1 to 256.
    #[must_use]
    pub fn palette_len(&self) -> u16 {
        todo!("P1: read the palette count")
    }

    /// The palette entry at `index`, or `None` at or beyond the palette count.
    #[must_use]
    pub fn palette_entry(&self, index: u8) -> Option<Rgba> {
        todo!("P1: read a palette entry")
    }

    /// The title, possibly empty.
    #[must_use]
    pub fn title(&self) -> &'a str {
        todo!("P1: read the title")
    }

    /// The number of tags.
    #[must_use]
    pub fn tag_count(&self) -> u16 {
        todo!("P1: read the tag count")
    }

    /// The tag at `index`, or `None` at or beyond the tag count.
    #[must_use]
    pub fn tag(&self, index: u16) -> Option<Tag<'a>> {
        todo!("P1: read a tag")
    }

    /// The number of frames, at least 1.
    #[must_use]
    pub fn frame_count(&self) -> u16 {
        todo!("P1: read the frame count")
    }

    /// The frames, in order.
    pub fn frames(&self) -> impl Iterator<Item = Frame<'a>> + 'a {
        core::iter::from_fn(|| todo!("P1: iterate over the frames"))
    }
}
