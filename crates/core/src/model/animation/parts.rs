use std::collections::BTreeMap;

use crate::error::DocumentError;
use crate::model::{Frame, Layer, Name, Palette, Tag};

use super::Animation;

/// Everything of an animation but its cels, before its rules are checked.
pub(crate) struct AnimationParts {
    pub(crate) title: Name,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) palette: Palette,
    pub(crate) layers: Vec<Layer>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) tags: Vec<Tag>,
    pub(crate) next_id: u32,
}

impl Animation {
    /// The animation of `parts`, without any cel: add them with [`Animation::insert_cel`].
    pub(crate) fn from_parts(parts: AnimationParts) -> Result<Self, DocumentError> {
        let animation = Self {
            title: parts.title,
            width: parts.width,
            height: parts.height,
            palette: parts.palette,
            layers: parts.layers,
            frames: parts.frames,
            cels: BTreeMap::new(),
            tags: parts.tags,
            next_id: parts.next_id,
        };
        animation.check_skeleton()?;
        Ok(animation)
    }
}
