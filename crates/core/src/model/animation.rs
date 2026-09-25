mod new_animation;
mod parts;
mod validate;

use std::collections::BTreeMap;

pub use new_animation::NewAnimation;
pub(crate) use parts::AnimationParts;

use super::{Cel, CelShape, Frame, FrameId, Layer, LayerId, Name, Palette, Tag};

/// A pixel-art animation, always valid: it is only built by [`Animation::new`], by reading a
/// document, or changed by the editing operations, each of which keeps every rule of the model.
///
/// Fields stay private to the crate; everything is read through the getters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animation {
    pub(crate) title: Name,
    pub(crate) width: u16,
    pub(crate) height: u16,
    /// Entry 0 is transparent, always.
    pub(crate) palette: Palette,
    /// From bottom to top.
    pub(crate) layers: Vec<Layer>,
    /// In play order.
    pub(crate) frames: Vec<Frame>,
    /// Non-blank cels only.
    pub(crate) cels: BTreeMap<(LayerId, FrameId), Cel>,
    pub(crate) tags: Vec<Tag>,
    /// Above every layer and frame id: new layers and frames take their ids from it.
    pub(crate) next_id: u32,
}

impl Animation {
    /// The title.
    #[must_use]
    pub fn title(&self) -> &Name {
        &self.title
    }

    /// The canvas width, in pixels.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The canvas height, in pixels.
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// The palette; entry 0 is transparent.
    #[must_use]
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// The layers, from bottom to top.
    #[must_use]
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// The frames, in play order.
    #[must_use]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    /// The non-blank cels, sorted by layer id then frame id.
    #[must_use]
    pub fn cels(&self) -> &BTreeMap<(LayerId, FrameId), Cel> {
        &self.cels
    }

    /// The tags.
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// The id the next layer or frame takes.
    #[must_use]
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// The layer `id`, when the animation has it.
    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id() == id)
    }

    /// The frame `id`, when the animation has it.
    #[must_use]
    pub fn frame(&self, id: FrameId) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.id() == id)
    }

    /// The cel of `layer` on `frame`; `None` when it is blank.
    #[must_use]
    pub fn cel(&self, layer: LayerId, frame: FrameId) -> Option<&Cel> {
        self.cels.get(&(layer, frame))
    }

    /// What every cel of this animation fits: its canvas and palette sizes.
    #[must_use]
    pub fn cel_shape(&self) -> CelShape {
        CelShape {
            width: self.width,
            height: self.height,
            palette_len: self.palette.len(),
        }
    }
}
