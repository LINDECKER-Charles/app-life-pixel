//! `preview`: a scaled PNG of one frame, or a contact sheet of every frame, so that an agent sees
//! what it drew — within `PREVIEW_MAX_SIDE` and `PREVIEW_MAX_BYTES`.

mod image;
mod layout;

use life_pixel_core::Animation;
use life_pixel_core::limits::PREVIEW_MAX_BYTES;
use serde::Serialize;

use super::addressing::frame_at;
use super::change::blocking;
use super::{AnimationEditing, EditingError};
use crate::ids::AnimationId;
use crate::owner::Owner;
use layout::Layout;

/// What `preview` renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewRequest {
    /// The animation.
    pub id: AnimationId,
    /// The frame's position, from 0; `None` for a contact sheet of every frame.
    pub frame: Option<u16>,
    /// How many times each pixel is repeated across and down; `None` for the automatic scale.
    pub scale: Option<u8>,
}

/// A preview: the PNG, and what it shows where.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Preview {
    /// The PNG file, at most `PREVIEW_MAX_BYTES`.
    #[serde(skip)]
    pub png: Vec<u8>,
    /// Its width, in pixels.
    pub width: u32,
    /// Its height, in pixels.
    pub height: u32,
    /// The scale it was rendered at.
    pub scale: u32,
    /// Where each frame lies, row by row.
    pub layout: Vec<PreviewCell>,
}

/// Where one frame lies on a preview, in the preview's pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PreviewCell {
    /// The frame's position, from 0.
    pub frame: u16,
    /// The column of its left edge.
    pub x: u32,
    /// The row of its top edge.
    pub y: u32,
    /// Its width.
    pub width: u32,
    /// Its height.
    pub height: u32,
}

impl AnimationEditing {
    /// A PNG of one frame's composite, or a contact sheet of all frames in `ceil(√n)` columns
    /// with a 2-pixel transparent gap, scaled. Without a scale, the largest up to 8 that keeps
    /// the longest side at most 512 pixels.
    ///
    /// # Errors
    ///
    /// `preview.too_large` beyond `PREVIEW_MAX_SIDE` or `PREVIEW_MAX_BYTES`; `export.scale` for a
    /// scale out of the export's bounds; `edit.frame_not_found`; the library's codes.
    pub async fn preview(
        &self,
        owner: &Owner,
        request: PreviewRequest,
    ) -> Result<Preview, EditingError> {
        let (_, animation) = self.read(owner, request.id).await?;
        blocking(move || render(&animation, request)).await
    }
}

fn render(animation: &Animation, request: PreviewRequest) -> Result<Preview, EditingError> {
    let positions = match request.frame {
        Some(position) => frame_at(animation, position).map(|_| vec![position])?,
        None => (0..=u16::MAX).take(animation.frames().len()).collect(),
    };
    let cell = (u32::from(animation.width()), u32::from(animation.height()));
    let layout = Layout::new(positions.len(), cell.0, cell.1);
    let scale = layout.scale(request.scale)?;
    let indices = image::draw(animation, &layout, &positions)?;
    let scaled = image::upscale(&indices, layout.width, scale);
    let size = (layout.width * scale, layout.height * scale);
    let png = within_bytes(image::encode(animation.palette(), size, &scaled)?)?;
    Ok(Preview {
        png,
        width: size.0,
        height: size.1,
        scale,
        layout: layout.cells(&positions, scale),
    })
}

/// `png` itself, when it holds at most [`PREVIEW_MAX_BYTES`].
fn within_bytes(png: Vec<u8>) -> Result<Vec<u8>, EditingError> {
    if png.len() > PREVIEW_MAX_BYTES {
        return Err(EditingError::PreviewTooLarge);
    }
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preview_holds_at_most_the_byte_cap() {
        assert!(within_bytes(vec![0; PREVIEW_MAX_BYTES]).is_ok());
        let above = within_bytes(vec![0; PREVIEW_MAX_BYTES + 1]);
        assert_eq!(above, Err(EditingError::PreviewTooLarge));
    }
}
