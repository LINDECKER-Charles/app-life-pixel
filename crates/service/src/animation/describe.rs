//! `describe`: an animation's view, with the composites of the frames asked for as text grids.

use life_pixel_core::Animation;
use life_pixel_core::render::composite;
use life_pixel_core::serialize::grid;

use super::addressing::frame_at;
use super::change::blocking;
use super::{AnimationEditing, AnimationView, EditingError, FrameGrid};
use crate::ids::AnimationId;
use crate::owner::Owner;

/// What `describe` reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescribeRequest {
    /// The animation.
    pub id: AnimationId,
    /// The positions of the frames whose composites to return as text grids; `None` returns no
    /// pixel.
    pub pixels: Option<Vec<u16>>,
}

impl AnimationEditing {
    /// The view of an animation: title, size, palette, layers, frames with their durations and
    /// tags; with `pixels`, those frames' composites as text grids.
    ///
    /// # Errors
    ///
    /// `edit.frame_not_found` for a position beyond the frames; `library.animation_not_found`;
    /// `service.unavailable`.
    pub async fn describe(
        &self,
        owner: &Owner,
        request: DescribeRequest,
    ) -> Result<AnimationView, EditingError> {
        let (record, animation) = self.read(owner, request.id).await?;
        let positions = request.pixels.unwrap_or_default();
        let view = blocking(move || {
            let grids = grids(&animation, &positions)?;
            Ok(AnimationView {
                grids,
                ..AnimationView::of(&record, &animation)
            })
        });
        view.await
    }
}

/// The composites of the frames at `positions`, in the order asked.
fn grids(animation: &Animation, positions: &[u16]) -> Result<Vec<FrameGrid>, EditingError> {
    let shape = animation.cel_shape();
    positions
        .iter()
        .map(|&position| {
            let frame = frame_at(animation, position)?;
            let indices = composite(animation, frame);
            Ok(FrameGrid {
                frame: position,
                rows: grid::format(&indices, shape),
            })
        })
        .collect()
}
