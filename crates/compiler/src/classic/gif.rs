//! GIF: the animation's palette as the global colour table, index 0 transparent.

use std::borrow::Cow;

use gif::{DisposalMethod, Encoder, Frame, Repeat};
use life_pixel_core::{Animation, LoopMode, Palette};

use super::plan::Plan;
use super::{ClassicOptions, indexed_png};
use crate::{ExportError, ExportFile};

/// The media type of a GIF.
const GIF_MEDIA_TYPE: &str = "image/gif";
/// The palette entry every exported GIF shows as transparent.
const TRANSPARENT_INDEX: u8 = 0;
/// The shortest delay browsers honour, in hundredths of a second: they slow down shorter ones.
const MIN_DELAY_CENTISECONDS: u32 = 2;
/// Milliseconds per hundredth of a second.
const MILLISECONDS_PER_CENTISECOND: u32 = 10;

/// The animation as a GIF, `<stem>.gif`: every frame full size, disposed to the background,
/// delayed by `max(2, round(ms / 10))` hundredths of a second; a looping range carries the
/// NETSCAPE loop extension with an infinite count, a range played once none. GIF has no partial
/// transparency: a colour with some alpha shows opaque, one without any shows transparent.
///
/// # Errors
///
/// [`ExportError::Scale`], [`ExportError::TagNotFound`] or [`ExportError::TooLarge`] when
/// `options` do not fit the animation.
pub fn export_gif(
    animation: &Animation,
    options: &ClassicOptions,
) -> Result<ExportFile, ExportError> {
    let plan = Plan::new(animation, options)?;
    let mut encoder = encoder(&plan)?;
    let transparency = transparency_map(plan.palette());
    for frame in plan.range.frames {
        encoder
            .write_frame(&gif_frame(&plan, frame, &transparency)?)
            .map_err(ExportError::encoding)?;
    }
    let bytes = encoder.into_inner().map_err(ExportError::encoding)?;
    Ok(ExportFile {
        name: format!("{}.gif", plan.stem()),
        media_type: GIF_MEDIA_TYPE,
        bytes,
    })
}

/// An encoder with the plan's size and palette, and the loop extension of a looping range.
fn encoder(plan: &Plan) -> Result<Encoder<Vec<u8>>, ExportError> {
    let (width, height) = frame_size(plan)?;
    let palette = indexed_png::rgb(plan.palette());
    let mut encoder =
        Encoder::new(Vec::new(), width, height, &palette).map_err(ExportError::encoding)?;
    if plan.range.loop_mode == LoopMode::Loop {
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(ExportError::encoding)?;
    }
    Ok(encoder)
}

/// `frame` full size, transparent at [`TRANSPARENT_INDEX`], restored to the background after.
fn gif_frame(
    plan: &Plan,
    frame: &life_pixel_core::Frame,
    transparency: &[u8],
) -> Result<Frame<'static>, ExportError> {
    let (width, height) = frame_size(plan)?;
    let indices = plan.render(frame);
    let buffer = indices
        .iter()
        .map(|&index| shown_index(transparency, index));
    Ok(Frame {
        width,
        height,
        buffer: Cow::Owned(buffer.collect()),
        delay: delay_centiseconds(frame.duration_ms()),
        dispose: DisposalMethod::Background,
        transparent: Some(TRANSPARENT_INDEX),
        ..Frame::default()
    })
}

/// A frame's size, which [`EXPORT_MAX_SIDE`](life_pixel_core::limits::EXPORT_MAX_SIDE) keeps
/// within GIF's 16 bits.
fn frame_size(plan: &Plan) -> Result<(u16, u16), ExportError> {
    let width = u16::try_from(plan.width).map_err(|_| ExportError::TooLarge)?;
    let height = u16::try_from(plan.height).map_err(|_| ExportError::TooLarge)?;
    Ok((width, height))
}

/// The index each palette entry is written as: [`TRANSPARENT_INDEX`] for a fully transparent
/// colour, since GIF has a single transparent entry, and the entry itself otherwise.
fn transparency_map(palette: &Palette) -> Vec<u8> {
    let entries = palette.entries().iter().zip(0..=u8::MAX);
    entries
        .map(|(colour, index)| {
            if colour.is_transparent() {
                TRANSPARENT_INDEX
            } else {
                index
            }
        })
        .collect()
}

/// The index `index` is written as; one outside the palette, which a valid animation never
/// holds, shows transparent.
fn shown_index(transparency: &[u8], index: u8) -> u8 {
    let shown = transparency.get(usize::from(index)).copied();
    shown.unwrap_or(TRANSPARENT_INDEX)
}

/// `max(2, round(duration_ms / 10))`, halves rounded up.
fn delay_centiseconds(duration_ms: u16) -> u16 {
    let halfway = MILLISECONDS_PER_CENTISECOND / 2;
    let rounded = (u32::from(duration_ms) + halfway) / MILLISECONDS_PER_CENTISECOND;
    let delay = rounded.max(MIN_DELAY_CENTISECONDS);
    u16::try_from(delay).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_round_to_hundredths_and_never_go_under_2() {
        assert_eq!(delay_centiseconds(10), 2);
        assert_eq!(delay_centiseconds(24), 2);
        assert_eq!(delay_centiseconds(25), 3);
        assert_eq!(delay_centiseconds(100), 10);
        assert_eq!(delay_centiseconds(u16::MAX), 6_554);
    }
}
