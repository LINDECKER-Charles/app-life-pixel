//! The rules an animation keeps, in the order a document is checked: canvas, layers, frames,
//! ids, tags, then each cel and the pixel budget.

use std::collections::BTreeSet;

use crate::error::DocumentError;
use crate::limits::{
    CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, MAX_CEL_PIXELS, MAX_FRAMES, MAX_LAYERS, MAX_TAGS,
};
use crate::model::{Cel, CelShape, FrameId, LayerId};

use super::Animation;

/// The fewest layers or frames of an animation.
const MIN_COUNT: usize = 1;

impl Animation {
    /// Checks every rule but those of the cels.
    pub(super) fn check_skeleton(&self) -> Result<(), DocumentError> {
        check_canvas(self.width, self.height)?;
        check_count(self.layers.len(), MAX_LAYERS, DocumentError::LayerCount)?;
        check_count(self.frames.len(), MAX_FRAMES, DocumentError::FrameCount)?;
        self.check_ids()?;
        self.check_tags()
    }

    /// Adds the cel of `key` after checking it: on an existing layer and frame, not there yet,
    /// fitting the cel shape and within the pixel budget. A blank cel is checked, then dropped.
    pub(crate) fn insert_cel(
        &mut self,
        key: (LayerId, FrameId),
        cel: Cel,
    ) -> Result<(), DocumentError> {
        let (layer, frame) = key;
        let is_referenced = self.layer(layer).is_some() && self.frame(frame).is_some();
        if !is_referenced || self.cels.contains_key(&key) {
            return Err(DocumentError::Reference);
        }
        if !self.cel_shape().fits(&cel) {
            return Err(DocumentError::Cel);
        }
        if cel.is_blank() {
            return Ok(());
        }
        check_pixel_budget(self.cels.len() + 1, self.cel_shape())?;
        self.cels.insert(key, cel);
        Ok(())
    }

    /// Checks every rule after an edit: the skeleton, the cels of `keys` — every cel when `keys`
    /// is `None` —, then the pixel budget.
    pub(crate) fn check_edited(
        &self,
        keys: Option<&[(LayerId, FrameId)]>,
    ) -> Result<(), DocumentError> {
        self.check_skeleton()?;
        match keys {
            Some(keys) => keys.iter().try_for_each(|key| self.check_stored_cel(*key)),
            None => self
                .cels
                .keys()
                .try_for_each(|key| self.check_stored_cel(*key)),
        }?;
        check_pixel_budget(self.cels.len(), self.cel_shape())
    }

    /// Whether `cel_count` non-blank cels would stay within the pixel budget.
    pub(crate) fn check_cel_count(&self, cel_count: usize) -> Result<(), DocumentError> {
        check_pixel_budget(cel_count, self.cel_shape())
    }

    /// The cel of `key`, when there is one, is on an existing layer and frame and fits.
    fn check_stored_cel(&self, key: (LayerId, FrameId)) -> Result<(), DocumentError> {
        let Some(cel) = self.cels.get(&key) else {
            return Ok(());
        };
        let (layer, frame) = key;
        if self.layer(layer).is_none() || self.frame(frame).is_none() {
            return Err(DocumentError::Reference);
        }
        self.cel_shape()
            .fits(cel)
            .then_some(())
            .ok_or(DocumentError::Cel)
    }

    /// Layer and frame ids are unique together, and below `next_id`.
    fn check_ids(&self) -> Result<(), DocumentError> {
        let layer_ids = self.layers.iter().map(|layer| layer.id().get());
        let frame_ids = self.frames.iter().map(|frame| frame.id().get());
        let ids: Vec<u32> = layer_ids.chain(frame_ids).collect();
        let unique: BTreeSet<u32> = ids.iter().copied().collect();
        let is_below_next = unique.last().is_none_or(|&highest| highest < self.next_id);
        (unique.len() == ids.len() && is_below_next)
            .then_some(())
            .ok_or(DocumentError::Reference)
    }

    /// At most [`MAX_TAGS`] tags, each inside the frames, with a unique name.
    fn check_tags(&self) -> Result<(), DocumentError> {
        if self.tags.len() > MAX_TAGS {
            return Err(DocumentError::TagCount);
        }
        let mut names = BTreeSet::new();
        let broken = self
            .tags
            .iter()
            .find(|tag| !tag.fits(self.frames.len()) || !names.insert(tag.name()));
        broken.map_or(Ok(()), |tag| {
            Err(DocumentError::Tag {
                name: tag.name().to_string(),
            })
        })
    }
}

fn check_canvas(width: u16, height: u16) -> Result<(), DocumentError> {
    let sides = CANVAS_MIN_SIDE..=CANVAS_MAX_SIDE;
    (sides.contains(&width) && sides.contains(&height))
        .then_some(())
        .ok_or(DocumentError::CanvasSize)
}

fn check_count(count: usize, max: usize, error: DocumentError) -> Result<(), DocumentError> {
    (MIN_COUNT..=max)
        .contains(&count)
        .then_some(())
        .ok_or(error)
}

fn check_pixel_budget(cel_count: usize, shape: CelShape) -> Result<(), DocumentError> {
    let pixels = cel_count.saturating_mul(shape.pixel_count());
    (pixels <= MAX_CEL_PIXELS)
        .then_some(())
        .ok_or(DocumentError::PixelBudget)
}
