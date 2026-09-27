//! The frames an export plays, and how it plays them.

use life_pixel_core::{Animation, Frame, LoopMode};

use crate::ExportError;

/// A run of consecutive frames of the animation.
pub(crate) struct Range<'animation> {
    /// The frames, in play order.
    pub frames: &'animation [Frame],
    /// The position of the first one in the animation.
    pub first: usize,
    /// How the range plays at its end.
    pub loop_mode: LoopMode,
}

impl<'animation> Range<'animation> {
    /// The frames of the tag named `tag`, played as it plays; every frame, looping, without one.
    pub fn select(
        animation: &'animation Animation,
        tag: Option<&str>,
    ) -> Result<Self, ExportError> {
        let Some(name) = tag else {
            return Ok(Self {
                frames: animation.frames(),
                first: 0,
                loop_mode: LoopMode::Loop,
            });
        };
        let not_found = || ExportError::TagNotFound {
            name: name.to_owned(),
        };
        let tag = animation
            .tags()
            .iter()
            .find(|tag| tag.name().as_str() == name);
        let tag = tag.ok_or_else(not_found)?;
        let first = usize::from(tag.first());
        let frames = animation.frames().get(first..=usize::from(tag.last()));
        Ok(Self {
            frames: frames.ok_or_else(not_found)?,
            first,
            loop_mode: tag.loop_mode(),
        })
    }

    /// Whether the positions `first..=last` of the animation all lie in the range.
    pub fn holds(&self, first: usize, last: usize) -> bool {
        first >= self.first && last < self.first + self.frames.len()
    }
}
