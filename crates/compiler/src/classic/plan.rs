//! An export's plan: its checked options, and its frames rendered at their scaled size.

use life_pixel_core::{Animation, Frame, Palette, render};

use super::indexed_png::IndexedImage;
use super::range::Range;
use super::{ClassicOptions, scale};
use crate::{ExportError, file_stem};

/// What an export draws: the frames of its range, `width × height` pixels each once scaled.
pub(crate) struct Plan<'animation> {
    animation: &'animation Animation,
    /// The frames exported, and how they play.
    pub range: Range<'animation>,
    /// How many times each pixel is repeated across and down.
    pub scale: u8,
    /// A frame's width once scaled.
    pub width: u32,
    /// A frame's height once scaled.
    pub height: u32,
}

impl<'animation> Plan<'animation> {
    /// Checks `options` against `animation`: the scale first, then the tag, then the size.
    pub fn new(
        animation: &'animation Animation,
        options: &ClassicOptions,
    ) -> Result<Self, ExportError> {
        let scale = scale::checked(options.scale)?;
        let range = Range::select(animation, options.tag.as_deref())?;
        let width = scale::bounded_side(u32::from(animation.width()), u32::from(scale))?;
        let height = scale::bounded_side(u32::from(animation.height()), u32::from(scale))?;
        Ok(Self {
            animation,
            range,
            scale,
            width,
            height,
        })
    }

    /// The palette indices of `frame`, composited by `life-pixel-core`, then scaled.
    pub fn render(&self, frame: &Frame) -> Vec<u8> {
        let indices = render::composite(self.animation, frame.id());
        let width = usize::from(self.animation.width());
        scale::upscale(&indices, width, usize::from(self.scale))
    }

    /// The animation's palette.
    pub fn palette(&self) -> &Palette {
        self.animation.palette()
    }

    /// The shape of one exported frame.
    pub fn frame_image(&self) -> IndexedImage<'_> {
        IndexedImage {
            palette: self.palette(),
            width: self.width,
            height: self.height,
        }
    }

    /// The animation exported.
    pub fn animation(&self) -> &Animation {
        self.animation
    }

    /// The stem of the files, from the animation's title.
    pub fn stem(&self) -> String {
        file_stem(self.animation.title().as_str())
    }
}
