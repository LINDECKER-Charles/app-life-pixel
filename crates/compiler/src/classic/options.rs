//! What a classic export exports: which frames, at which size.

/// The scale of an export at the animation's own size.
const NATURAL_SCALE: u8 = 1;

/// The options every classic export takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassicOptions {
    /// The tag whose frames to export, played as the tag plays; `None` exports every frame,
    /// looping.
    pub tag: Option<String>,
    /// How many times each pixel is repeated across and down, from
    /// [`EXPORT_MIN_SCALE`](life_pixel_core::limits::EXPORT_MIN_SCALE) to
    /// [`EXPORT_MAX_SCALE`](life_pixel_core::limits::EXPORT_MAX_SCALE).
    pub scale: u8,
}

impl Default for ClassicOptions {
    /// Every frame, at the animation's own size.
    fn default() -> Self {
        Self {
            tag: None,
            scale: NATURAL_SCALE,
        }
    }
}
