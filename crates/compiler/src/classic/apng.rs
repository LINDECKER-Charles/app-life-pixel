//! Animated PNG: indexed colour with its full alpha, one full frame per frame.

use life_pixel_core::LoopMode;

use super::ClassicOptions;
use super::plan::Plan;
use crate::{ExportError, ExportFile};

/// The media type of an animated PNG.
const APNG_MEDIA_TYPE: &str = "image/apng";
/// Frame delays are written in milliseconds: `duration / 1000` seconds.
const MILLISECONDS_PER_SECOND: u16 = 1_000;
/// `num_plays` of a range that loops forever.
const PLAYS_FOREVER: u32 = 0;
/// `num_plays` of a range played once.
const PLAYS_ONCE: u32 = 1;

/// The animation as an APNG, `<stem>.apng`: `PLTE` and `tRNS` from its palette, each frame
/// delayed by its duration, played forever for a looping range and once otherwise.
///
/// # Errors
///
/// [`ExportError::Scale`], [`ExportError::TagNotFound`] or [`ExportError::TooLarge`] when
/// `options` do not fit the animation.
pub fn export_apng(
    animation: &life_pixel_core::Animation,
    options: &ClassicOptions,
) -> Result<ExportFile, ExportError> {
    let plan = Plan::new(animation, options)?;
    let frame_count = u32::try_from(plan.range.frames.len()).map_err(ExportError::encoding)?;
    let plays = match plan.range.loop_mode {
        LoopMode::Loop => PLAYS_FOREVER,
        LoopMode::Once => PLAYS_ONCE,
    };
    let mut bytes = Vec::new();
    let mut encoder = plan.frame_image().encoder(&mut bytes);
    encoder
        .set_animated(frame_count, plays)
        .map_err(ExportError::encoding)?;
    let mut writer = encoder.write_header().map_err(ExportError::encoding)?;
    for frame in plan.range.frames {
        writer
            .set_frame_delay(frame.duration_ms(), MILLISECONDS_PER_SECOND)
            .map_err(ExportError::encoding)?;
        writer
            .write_image_data(&plan.render(frame))
            .map_err(ExportError::encoding)?;
    }
    writer.finish().map_err(ExportError::encoding)?;
    Ok(ExportFile {
        name: format!("{}.apng", plan.stem()),
        media_type: APNG_MEDIA_TYPE,
        bytes,
    })
}
