//! Indexed PNGs: the animation's palette as `PLTE`, its alpha as `tRNS`, one byte per pixel.

use std::io::Write;

use life_pixel_core::Palette;
use png::{BitDepth, ColorType, Encoder};

use crate::ExportError;

/// The media type of a PNG.
pub(crate) const PNG_MEDIA_TYPE: &str = "image/png";

/// The shape of indexed images: their palette, and their size in pixels.
pub(crate) struct IndexedImage<'palette> {
    /// The palette the indices point into.
    pub palette: &'palette Palette,
    /// The width, in pixels.
    pub width: u32,
    /// The height, in pixels.
    pub height: u32,
}

impl IndexedImage<'_> {
    /// An encoder of such images: 8-bit indices, the palette's full alpha kept.
    pub fn encoder<W: Write>(&self, writer: W) -> Encoder<'static, W> {
        let mut encoder = Encoder::new(writer, self.width, self.height);
        encoder.set_color(ColorType::Indexed);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_palette(rgb(self.palette));
        encoder.set_trns(alpha(self.palette));
        encoder
    }

    /// One still image of `indices`, row by row.
    pub fn encode(&self, indices: &[u8]) -> Result<Vec<u8>, ExportError> {
        let mut bytes = Vec::new();
        let mut writer = self
            .encoder(&mut bytes)
            .write_header()
            .map_err(ExportError::encoding)?;
        writer
            .write_image_data(indices)
            .map_err(ExportError::encoding)?;
        writer.finish().map_err(ExportError::encoding)?;
        Ok(bytes)
    }
}

/// The palette's colours, 3 bytes each — red, green, blue —, alpha left out.
pub(crate) fn rgb(palette: &Palette) -> Vec<u8> {
    let colours = palette.entries().iter();
    colours
        .flat_map(|colour| [colour.r, colour.g, colour.b])
        .collect()
}

/// The palette's alpha, one byte per entry.
fn alpha(palette: &Palette) -> Vec<u8> {
    palette.entries().iter().map(|colour| colour.a).collect()
}
