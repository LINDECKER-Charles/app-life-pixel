//! A preview's pixels: `core`'s composites in their cells, each pixel repeated `scale` times,
//! encoded as an indexed PNG with the animation's palette.

use life_pixel_compiler::ExportError;
use life_pixel_core::render::composite;
use life_pixel_core::{Animation, Palette};
use png::{BitDepth, ColorType, Encoder};

use super::layout::Layout;
use crate::animation::EditingError;
use crate::animation::addressing::frame_at;

/// The palette indices of the unscaled preview: each frame's composite in its cell, index 0 —
/// transparent — in the gaps and empty cells.
pub(super) fn draw(
    animation: &Animation,
    layout: &Layout,
    positions: &[u16],
) -> Result<Vec<u8>, EditingError> {
    let sheet_width = to_usize(layout.width);
    let frame_width = usize::from(animation.width()).max(1);
    let mut sheet = vec![0; sheet_width * to_usize(layout.height)];
    for (slot, &position) in positions.iter().enumerate() {
        let frame = frame_at(animation, position)?;
        let (left, top) = layout.origin(slot);
        let rows = composite(animation, frame);
        for (row, pixels) in rows.chunks(frame_width).enumerate() {
            let start = (to_usize(top) + row) * sheet_width + to_usize(left);
            sheet[start..start + pixels.len()].copy_from_slice(pixels);
        }
    }
    Ok(sheet)
}

/// The pixels of an image `width` pixels wide, each repeated `scale` times across and down.
pub(super) fn upscale(pixels: &[u8], width: u32, scale: u32) -> Vec<u8> {
    let scale = to_usize(scale);
    let mut scaled = Vec::with_capacity(pixels.len() * scale * scale);
    for row in pixels.chunks(to_usize(width).max(1)) {
        let wide_row: Vec<u8> = row
            .iter()
            .flat_map(|&pixel| std::iter::repeat_n(pixel, scale))
            .collect();
        for _ in 0..scale {
            scaled.extend_from_slice(&wide_row);
        }
    }
    scaled
}

/// The PNG of `indices`, `width × height` pixels, with `palette` as `PLTE` and its alpha as
/// `tRNS`.
pub(super) fn encode(
    palette: &Palette,
    (width, height): (u32, u32),
    indices: &[u8],
) -> Result<Vec<u8>, EditingError> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Indexed);
    encoder.set_depth(BitDepth::Eight);
    let entries = palette.entries();
    encoder.set_palette(
        entries
            .iter()
            .flat_map(|c| [c.r, c.g, c.b])
            .collect::<Vec<u8>>(),
    );
    encoder.set_trns(entries.iter().map(|colour| colour.a).collect::<Vec<u8>>());
    let mut writer = encoder.write_header().map_err(encoding)?;
    writer.write_image_data(indices).map_err(encoding)?;
    writer.finish().map_err(encoding)?;
    Ok(bytes)
}

/// An encoder's failure, which valid indices within the caps never cause.
fn encoding(error: png::EncodingError) -> EditingError {
    ExportError::Encoding(error.to_string()).into()
}

/// A side within the preview's caps, as an index.
fn to_usize(side: u32) -> usize {
    usize::try_from(side).unwrap_or(usize::MAX)
}
