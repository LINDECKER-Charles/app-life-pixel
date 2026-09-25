//! The frames of a loaded payload: the frame table, the key frames, the palette indices of the
//! frame shown, and the framebuffer painted from them through the palette.

use alloc::vec::Vec;

use life_pixel_format::{Frame, FrameKind, Payload, apply_frame};

use crate::CallError;
use crate::memory::{collected, zeroed};
use crate::span::Span;

/// The bytes of a framebuffer pixel: red, green, blue, alpha.
const RGBA_BYTES: usize = 4;

/// A frame of the table: what `Frame` holds, its data as a span of the payload.
#[derive(Clone, Copy, Debug)]
struct FrameEntry {
    duration_ms: u16,
    kind: FrameKind,
    data: Span,
}

/// The frame table, the index buffer and the framebuffer of a loaded payload.
#[derive(Clone, Debug)]
pub(crate) struct Frames {
    table: Vec<FrameEntry>,
    palette: Vec<[u8; RGBA_BYTES]>,
    palette_len: u16,
    /// The palette indices of frame `decoded`, `width × height` of them.
    indices: Vec<u8>,
    /// The frame `indices` holds, if they hold one.
    decoded: Option<u16>,
    /// `width × height` pixels of RGBA, rows top to bottom: what `ImageData` expects.
    framebuffer: Vec<u8>,
}

impl Frames {
    /// Reserves the table and the buffers of `parsed`, parsed from `payload`; nothing is shown
    /// yet.
    pub(crate) fn new(payload: &[u8], parsed: &Payload<'_>) -> Result<Self, CallError> {
        let pixel_count = usize::from(parsed.width()) * usize::from(parsed.height());
        Ok(Self {
            table: read_table(payload, parsed)?,
            palette: read_palette(parsed)?,
            palette_len: parsed.palette_len(),
            indices: zeroed(pixel_count)?,
            decoded: None,
            framebuffer: zeroed(pixel_count * RGBA_BYTES)?,
        })
    }

    /// How long `frame` shows, in milliseconds: never 0.
    pub(crate) fn duration_ms(&self, frame: u16) -> u16 {
        self.entry(frame)
            .map_or(u16::MAX, |entry| entry.duration_ms)
    }

    /// The address of the framebuffer, which never moves.
    pub(crate) fn framebuffer_address(&self) -> usize {
        self.framebuffer.as_ptr().addr()
    }

    /// Decodes `frame` of `payload` and paints it into the framebuffer. From the frame decoded
    /// before when `frame` follows it, with no key frame between them — a single delta for the
    /// next frame —, else from the nearest key frame at or before `frame`.
    pub(crate) fn show(&mut self, payload: &[u8], frame: u16) {
        let first = self.first_to_apply(frame);
        let is_decoded = (first..=frame).all(|position| self.apply(payload, position));
        self.decoded = is_decoded.then_some(frame);
        self.paint();
    }

    fn first_to_apply(&self, frame: u16) -> u16 {
        let key = (0..=frame)
            .rev()
            .find(|&position| self.is_key(position))
            .unwrap_or(0);
        match self.decoded {
            Some(decoded) if key <= decoded && decoded < frame => decoded + 1,
            _ => key,
        }
    }

    fn is_key(&self, frame: u16) -> bool {
        self.entry(frame)
            .is_some_and(|entry| entry.kind == FrameKind::Key)
    }

    /// Applies the operations of frame `position` to the indices; `false` if they do not apply,
    /// which a checked payload never does.
    fn apply(&mut self, payload: &[u8], position: u16) -> bool {
        let Some(entry) = self.entry(position) else {
            return false;
        };
        let frame = Frame {
            duration_ms: entry.duration_ms,
            kind: entry.kind,
            data: entry.data.bytes(payload),
        };
        apply_frame(&frame, &mut self.indices, self.palette_len).is_ok()
    }

    fn paint(&mut self) {
        let (pixels, _) = self.framebuffer.as_chunks_mut::<RGBA_BYTES>();
        for (pixel, &index) in pixels.iter_mut().zip(&self.indices) {
            let rgba = self.palette.get(usize::from(index)).copied();
            *pixel = rgba.unwrap_or_default();
        }
    }

    fn entry(&self, frame: u16) -> Option<&FrameEntry> {
        self.table.get(usize::from(frame))
    }
}

/// The frame table of `parsed`: each frame's duration, kind, and where its data lies.
fn read_table(payload: &[u8], parsed: &Payload<'_>) -> Result<Vec<FrameEntry>, CallError> {
    let entries = parsed.frames().map(|frame| FrameEntry {
        duration_ms: frame.duration_ms,
        kind: frame.kind,
        data: Span::of(payload, frame.data),
    });
    collected(usize::from(parsed.frame_count()), entries)
}

/// The palette of `parsed`, entry by entry as the framebuffer takes it.
fn read_palette(parsed: &Payload<'_>) -> Result<Vec<[u8; RGBA_BYTES]>, CallError> {
    let entries = (0..=u8::MAX).map_while(|index| parsed.palette_entry(index));
    let entries = entries.map(|rgba| [rgba.r, rgba.g, rgba.b, rgba.a]);
    collected(usize::from(parsed.palette_len()), entries)
}

#[cfg(test)]
mod tests;
