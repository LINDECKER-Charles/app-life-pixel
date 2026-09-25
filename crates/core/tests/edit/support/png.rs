//! PNG files made for the import tests, in every colour type the import expands.

use png::{BitDepth, ColorType, Encoder};

/// What a PNG file holds: its size, its pixel format and data, and, for a paletted image, its
/// palette and the alpha of its entries.
pub struct PngSpec<'data> {
    pub width: u32,
    pub height: u32,
    pub color: ColorType,
    pub depth: BitDepth,
    pub data: &'data [u8],
    pub palette: Option<&'data [u8]>,
    pub transparency: Option<&'data [u8]>,
}

/// The PNG file of `spec`.
pub fn encode(spec: &PngSpec) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, spec.width, spec.height);
    encoder.set_color(spec.color);
    encoder.set_depth(spec.depth);
    if let Some(palette) = spec.palette {
        encoder.set_palette(palette);
    }
    if let Some(transparency) = spec.transparency {
        encoder.set_trns(transparency);
    }
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(spec.data).unwrap();
    writer.finish().unwrap();
    bytes
}

/// An 8-bit RGBA PNG file of `pixels`, row by row.
pub fn rgba(width: u32, height: u32, pixels: &[[u8; 4]]) -> Vec<u8> {
    let data: Vec<u8> = pixels.iter().flatten().copied().collect();
    encode(&PngSpec {
        width,
        height,
        color: ColorType::Rgba,
        depth: BitDepth::Eight,
        data: &data,
        palette: None,
        transparency: None,
    })
}

/// A PNG file whose header claims `width × height` pixels, with no pixel data at all.
pub fn header_only(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    drop(encoder.write_header().unwrap());
    bytes
}
