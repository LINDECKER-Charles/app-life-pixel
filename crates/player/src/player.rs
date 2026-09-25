use alloc::vec::Vec;
use core::mem;

use life_pixel_format::ABI_VERSION;
use life_pixel_format::bounds::MAX_PAYLOAD_BYTES;

use crate::CallError;
use crate::animation::Animation;
use crate::memory::zeroed;

/// Where an instance stands in the calls of player ABI v1: `alloc`, then `load`, then playback.
#[derive(Clone, Debug)]
enum Stage {
    /// Nothing called yet.
    Fresh,
    /// `alloc` reserved these bytes, which the loader fills with the payload.
    Reserved(Vec<u8>),
    /// `load` accepted the payload.
    Loaded(Animation),
    /// `alloc` or `load` failed: the instance plays nothing.
    Spent,
}

/// The player of one module instance: one method per export of player ABI v1, as
/// `crates/format/README.md` specifies them. Pointers are addresses; the exports narrow them to
/// the 32 bits of `wasm32`.
///
/// Before a successful `load`, a query returns `0` and a command [`CallError::WrongCallOrder`].
#[derive(Clone, Debug)]
pub struct Player {
    stage: Stage,
}

impl Player {
    /// An instance on which nothing was called yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stage: Stage::Fresh,
        }
    }

    /// `abi_version`: the ABI this player implements, `1`.
    #[must_use]
    pub fn abi_version(&self) -> u32 {
        u32::from(ABI_VERSION)
    }

    /// `alloc`: reserves `len` bytes for the payload, which the loader writes into. `None` —
    /// the export's `0` — on a second call, beyond `MAX_PAYLOAD_BYTES`, or out of memory.
    pub fn alloc(&mut self, len: u32) -> Option<&mut [u8]> {
        if !matches!(self.stage, Stage::Fresh) {
            return None;
        }
        self.stage = zeroed_block(len).map_or(Stage::Spent, Stage::Reserved);
        match &mut self.stage {
            Stage::Reserved(block) => Some(block),
            _ => None,
        }
    }

    /// `load`: checks the payload written at `address`, the block `alloc` returned, over its
    /// first `len` bytes, then shows the first frame of the initial range. A refused payload
    /// spends the instance; a block other than the reserved one is out of range.
    ///
    /// # Errors
    ///
    /// The [`CallError`] whose status the export returns.
    pub fn load(&mut self, address: usize, len: u32) -> Result<(), CallError> {
        let Stage::Reserved(block) = &mut self.stage else {
            return Err(CallError::WrongCallOrder);
        };
        let len = usize::try_from(len).map_err(|_| CallError::ArgumentOutOfRange)?;
        if block.as_ptr().addr() != address || len > block.len() {
            return Err(CallError::ArgumentOutOfRange);
        }
        block.truncate(len);
        let payload = mem::take(block);
        self.stage = Stage::Spent;
        self.stage = Stage::Loaded(Animation::load(payload)?);
        Ok(())
    }

    /// `width`: the canvas width, in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.animation().map_or(0, Animation::width)
    }

    /// `height`: the canvas height, in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.animation().map_or(0, Animation::height)
    }

    /// `frame_ptr`: the address of the framebuffer, `width × height × 4` bytes of RGBA.
    #[must_use]
    pub fn frame_ptr(&self) -> usize {
        self.animation().map_or(0, Animation::frame_address)
    }

    /// `tick`: advances playback by `elapsed_ms`; returns [`FRAME_CHANGED`](crate::FRAME_CHANGED),
    /// [`RANGE_ENDED`](crate::RANGE_ENDED) and [`RANGE_STOPPED`](crate::RANGE_STOPPED).
    pub fn tick(&mut self, elapsed_ms: u32) -> u32 {
        self.animation_mut()
            .map_or(0, |animation| animation.tick(elapsed_ms))
    }

    /// `tag_count`: the number of tags.
    #[must_use]
    pub fn tag_count(&self) -> u32 {
        self.animation().map_or(0, Animation::tag_count)
    }

    /// `tag_name_ptr`: the address of tag `index`'s name; `0` for an index out of range.
    #[must_use]
    pub fn tag_name_ptr(&self, index: u32) -> usize {
        self.tag_name(index).map_or(0, address)
    }

    /// `tag_name_len`: the length of tag `index`'s name, in bytes; `0` for an index out of
    /// range.
    #[must_use]
    pub fn tag_name_len(&self, index: u32) -> u32 {
        self.tag_name(index).map_or(0, length)
    }

    /// `set_tag`: plays tag `index`, or the whole animation for `0xFFFFFFFF`, from its first
    /// frame.
    ///
    /// # Errors
    ///
    /// The [`CallError`] whose status the export returns.
    pub fn set_tag(&mut self, index: u32) -> Result<(), CallError> {
        self.animation_mut()?.set_tag(index)
    }

    /// `set_loop`: `0` the range's own mode, `1` loop, `2` once; applies at once.
    ///
    /// # Errors
    ///
    /// The [`CallError`] whose status the export returns.
    pub fn set_loop(&mut self, mode: u32) -> Result<(), CallError> {
        self.animation_mut()?.set_loop(mode)
    }

    /// `seek`: shows frame `frame` of the current range, counted from its first frame, and
    /// restarts the range.
    ///
    /// # Errors
    ///
    /// The [`CallError`] whose status the export returns.
    pub fn seek(&mut self, frame: u32) -> Result<(), CallError> {
        self.animation_mut()?.seek(frame)
    }

    /// `frame_index`: the animation frame shown, counted from frame 0 of the animation.
    #[must_use]
    pub fn frame_index(&self) -> u32 {
        self.animation().map_or(0, Animation::frame_index)
    }

    /// `title_ptr`: the address of the UTF-8 title.
    #[must_use]
    pub fn title_ptr(&self) -> usize {
        self.animation()
            .map_or(0, |animation| address(animation.title()))
    }

    /// `title_len`: the length of the title, in bytes.
    #[must_use]
    pub fn title_len(&self) -> u32 {
        self.animation()
            .map_or(0, |animation| length(animation.title()))
    }

    fn tag_name(&self, index: u32) -> Option<&[u8]> {
        self.animation()?.tag_name(index)
    }

    fn animation(&self) -> Option<&Animation> {
        match &self.stage {
            Stage::Loaded(animation) => Some(animation),
            _ => None,
        }
    }

    fn animation_mut(&mut self) -> Result<&mut Animation, CallError> {
        match &mut self.stage {
            Stage::Loaded(animation) => Ok(animation),
            _ => Err(CallError::WrongCallOrder),
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

fn address(bytes: &[u8]) -> usize {
    bytes.as_ptr().addr()
}

fn length(bytes: &[u8]) -> u32 {
    u32::try_from(bytes.len()).unwrap_or(0)
}

/// `len` zero bytes for the payload; `None` beyond the largest payload or out of memory.
fn zeroed_block(len: u32) -> Option<Vec<u8>> {
    if len > MAX_PAYLOAD_BYTES {
        return None;
    }
    zeroed(usize::try_from(len).ok()?).ok()
}

#[cfg(test)]
mod tests;
