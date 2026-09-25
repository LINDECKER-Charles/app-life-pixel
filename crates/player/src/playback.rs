//! The playback rules of player ABI v1, pure: which range plays, which frame it shows, and how
//! time moves it. Durations come from the caller, so the rules are tested without a payload.

use life_pixel_format::LoopMode;

use crate::CallError;

/// `tick`'s bit 0: the frame shown changed, so the framebuffer did.
pub const FRAME_CHANGED: u32 = 1;

/// `tick`'s bit 1: playback left the range's last frame.
pub const RANGE_ENDED: u32 = 2;

/// `tick`'s bit 2: the range that ended stayed there, rather than looping back to its first
/// frame. Set together with [`RANGE_ENDED`], never alone: a loader cannot tell a range that
/// stopped from one that looped back to its first frame by comparing frame indices alone — a
/// looping range whose elapsed time is reduced modulo its total duration (see [`Playback::tick`])
/// may land anywhere in the range, including on its last frame, without having stopped. This bit
/// is the only reliable signal.
pub const RANGE_STOPPED: u32 = 4;

/// The frames a range plays, and how it ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Range {
    /// The range's first frame, counted from frame 0 of the animation.
    pub first: u16,
    /// The range's last frame, included.
    pub last: u16,
    /// The range's own mode: a tag's, or `Loop` for the whole animation.
    pub loop_mode: LoopMode,
}

/// What `set_loop` asks for: the range's own mode, or a mode that overrides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LoopSetting {
    /// The range's own mode: `set_loop(0)`, the default.
    Own,
    /// Loop, whatever the range's mode: `set_loop(1)`.
    Loop,
    /// Play once, whatever the range's mode: `set_loop(2)`.
    Once,
}

impl LoopSetting {
    /// The setting `set_loop(mode)` asks for.
    pub(crate) fn from_mode(mode: u32) -> Result<Self, CallError> {
        match mode {
            0 => Ok(Self::Own),
            1 => Ok(Self::Loop),
            2 => Ok(Self::Once),
            _ => Err(CallError::ArgumentOutOfRange),
        }
    }
}

/// Where playback stands: the range, the frame shown, and the time spent on it.
#[derive(Clone, Debug)]
pub(crate) struct Playback {
    range: Range,
    loop_setting: LoopSetting,
    /// The range's total duration: at least 1 ms, since every frame lasts at least that.
    total_ms: u32,
    frame: u16,
    /// The time spent on `frame`, always below its duration.
    time_ms: u32,
    is_stopped: bool,
}

impl Playback {
    /// Plays `range` from its first frame, in its own mode.
    pub(crate) fn new(range: Range, duration_ms: impl Fn(u16) -> u16) -> Self {
        Self {
            range,
            loop_setting: LoopSetting::Own,
            total_ms: total_ms(range, duration_ms),
            frame: range.first,
            time_ms: 0,
            is_stopped: false,
        }
    }

    /// The animation frame shown, counted from frame 0 of the animation.
    pub(crate) fn frame(&self) -> u16 {
        self.frame
    }

    /// Plays `range` from its first frame; the loop setting stays.
    pub(crate) fn set_range(&mut self, range: Range, duration_ms: impl Fn(u16) -> u16) {
        self.range = range;
        self.total_ms = total_ms(range, duration_ms);
        self.restart_on(range.first);
    }

    /// Sets how the range ends, from its next end on; a stopped range stays stopped.
    pub(crate) fn set_loop(&mut self, setting: LoopSetting) {
        self.loop_setting = setting;
    }

    /// Shows the frame `offset` frames after the range's first one, and restarts the range.
    pub(crate) fn seek(&mut self, offset: u32) -> Result<(), CallError> {
        let length = u32::from(self.range.last - self.range.first);
        let offset = u16::try_from(offset)
            .ok()
            .filter(|&offset| u32::from(offset) <= length)
            .ok_or(CallError::ArgumentOutOfRange)?;
        self.restart_on(self.range.first + offset);
        Ok(())
    }

    /// Advances playback by `elapsed_ms` and returns what changed: [`FRAME_CHANGED`],
    /// [`RANGE_ENDED`] and [`RANGE_STOPPED`]. A looping range first drops whole cycles, so a long
    /// pause costs one cycle at most; a range played once stops on its last frame, and later
    /// ticks return `0`.
    pub(crate) fn tick(&mut self, elapsed_ms: u32, duration_ms: impl Fn(u16) -> u16) -> u32 {
        if self.is_stopped {
            return 0;
        }
        let shown = self.frame;
        let mut flags = 0;
        let mut elapsed_ms = elapsed_ms;
        if self.is_looping() && elapsed_ms >= self.total_ms {
            elapsed_ms = elapsed_ms.checked_rem(self.total_ms).unwrap_or(0);
            flags |= RANGE_ENDED;
        }
        self.time_ms = self.time_ms.saturating_add(elapsed_ms);
        while !self.is_stopped {
            let duration_ms = u32::from(duration_ms(self.frame));
            if self.time_ms < duration_ms {
                break;
            }
            self.time_ms -= duration_ms;
            flags |= self.leave_frame();
        }
        if self.frame != shown {
            flags |= FRAME_CHANGED;
        }
        flags
    }

    /// Leaves the frame shown for the next one, or ends the range: back to its first frame when
    /// it loops, stopped on its last one otherwise. Returns [`RANGE_ENDED`] when the range ended,
    /// plus [`RANGE_STOPPED`] when it stopped there rather than looping.
    fn leave_frame(&mut self) -> u32 {
        if self.frame < self.range.last {
            self.frame += 1;
            return 0;
        }
        if self.is_looping() {
            self.frame = self.range.first;
            return RANGE_ENDED;
        }
        self.is_stopped = true;
        self.time_ms = 0;
        RANGE_ENDED | RANGE_STOPPED
    }

    fn is_looping(&self) -> bool {
        match self.loop_setting {
            LoopSetting::Own => self.range.loop_mode == LoopMode::Loop,
            LoopSetting::Loop => true,
            LoopSetting::Once => false,
        }
    }

    fn restart_on(&mut self, frame: u16) {
        self.frame = frame;
        self.time_ms = 0;
        self.is_stopped = false;
    }
}

/// The sum of the durations of `range`'s frames.
fn total_ms(range: Range, duration_ms: impl Fn(u16) -> u16) -> u32 {
    (range.first..=range.last)
        .map(|frame| u32::from(duration_ms(frame)))
        .sum()
}

#[cfg(test)]
mod tests;
