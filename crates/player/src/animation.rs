use alloc::vec::Vec;

use life_pixel_format::{LoopMode, Payload};

use crate::CallError;
use crate::frames::Frames;
use crate::memory::collected;
use crate::playback::{FRAME_CHANGED, LoopSetting, Playback, Range};
use crate::span::Span;

/// `set_tag`'s index for the whole animation.
const WHOLE_ANIMATION: u32 = u32::MAX;

/// A tag of the payload: its range, and where its name lies.
#[derive(Clone, Copy, Debug)]
struct TagEntry {
    range: Range,
    name: Span,
}

/// A payload `load` accepted: its bytes, its frames, its tags and where playback stands. It
/// holds everything playback needs, reserved once.
#[derive(Clone, Debug)]
pub(crate) struct Animation {
    payload: Vec<u8>,
    width: u16,
    height: u16,
    frame_count: u16,
    title: Span,
    tags: Vec<TagEntry>,
    frames: Frames,
    playback: Playback,
}

impl Animation {
    /// Checks the whole payload, reserves what it needs, then shows the first frame of the
    /// initial range: tag 0 when there are tags, the whole animation otherwise.
    pub(crate) fn load(payload: Vec<u8>) -> Result<Self, CallError> {
        let parsed = Payload::parse(&payload).map_err(CallError::Payload)?;
        let frames = Frames::new(&payload, &parsed)?;
        let tags = read_tags(&payload, &parsed)?;
        let frame_count = parsed.frame_count();
        let initial = tags
            .first()
            .map_or_else(|| whole_range(frame_count), |tag| tag.range);
        let mut animation = Self {
            width: parsed.width(),
            height: parsed.height(),
            frame_count,
            title: Span::of(&payload, parsed.title().as_bytes()),
            tags,
            playback: Playback::new(initial, |frame| frames.duration_ms(frame)),
            frames,
            payload,
        };
        animation.show();
        Ok(animation)
    }

    /// The canvas width, in pixels.
    pub(crate) fn width(&self) -> u32 {
        u32::from(self.width)
    }

    /// The canvas height, in pixels.
    pub(crate) fn height(&self) -> u32 {
        u32::from(self.height)
    }

    /// The address of the framebuffer.
    pub(crate) fn frame_address(&self) -> usize {
        self.frames.framebuffer_address()
    }

    /// Advances playback, repainting when the frame shown changes; returns `tick`'s flags.
    pub(crate) fn tick(&mut self, elapsed_ms: u32) -> u32 {
        let frames = &self.frames;
        let flags = self
            .playback
            .tick(elapsed_ms, |frame| frames.duration_ms(frame));
        if flags & FRAME_CHANGED != 0 {
            self.show();
        }
        flags
    }

    /// The number of tags.
    pub(crate) fn tag_count(&self) -> u32 {
        u32::try_from(self.tags.len()).unwrap_or(0)
    }

    /// The UTF-8 name of tag `index`, if there is such a tag.
    pub(crate) fn tag_name(&self, index: u32) -> Option<&[u8]> {
        self.tag(index).map(|tag| tag.name.bytes(&self.payload))
    }

    /// Plays tag `index`, or the whole animation for [`WHOLE_ANIMATION`], from its first frame.
    pub(crate) fn set_tag(&mut self, index: u32) -> Result<(), CallError> {
        let range = match index {
            WHOLE_ANIMATION => whole_range(self.frame_count),
            index => self.tag(index).ok_or(CallError::ArgumentOutOfRange)?.range,
        };
        let frames = &self.frames;
        self.playback
            .set_range(range, |frame| frames.duration_ms(frame));
        self.show();
        Ok(())
    }

    /// Sets how ranges end: `0` their own mode, `1` loop, `2` once.
    pub(crate) fn set_loop(&mut self, mode: u32) -> Result<(), CallError> {
        self.playback.set_loop(LoopSetting::from_mode(mode)?);
        Ok(())
    }

    /// Shows frame `frame` of the current range, counted from its first frame.
    pub(crate) fn seek(&mut self, frame: u32) -> Result<(), CallError> {
        self.playback.seek(frame)?;
        self.show();
        Ok(())
    }

    /// The animation frame shown.
    pub(crate) fn frame_index(&self) -> u32 {
        u32::from(self.playback.frame())
    }

    /// The UTF-8 title.
    pub(crate) fn title(&self) -> &[u8] {
        self.title.bytes(&self.payload)
    }

    fn tag(&self, index: u32) -> Option<&TagEntry> {
        self.tags.get(usize::try_from(index).ok()?)
    }

    fn show(&mut self) {
        self.frames.show(&self.payload, self.playback.frame());
    }
}

/// The whole animation as a range: it loops.
fn whole_range(frame_count: u16) -> Range {
    Range {
        first: 0,
        last: frame_count.saturating_sub(1),
        loop_mode: LoopMode::Loop,
    }
}

/// The tags of `parsed`, parsed from `payload`.
fn read_tags(payload: &[u8], parsed: &Payload<'_>) -> Result<Vec<TagEntry>, CallError> {
    let tags = (0..parsed.tag_count()).filter_map(|index| parsed.tag(index));
    let entries = tags.map(|tag| TagEntry {
        range: Range {
            first: tag.first,
            last: tag.last,
            loop_mode: tag.loop_mode,
        },
        name: Span::of(payload, tag.name.as_bytes()),
    });
    collected(usize::from(parsed.tag_count()), entries)
}
