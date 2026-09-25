//! A PNG file read through the `png` crate, under the import limits: its size is checked before
//! reading, its dimensions right after the header — before any pixel is decoded —, then it is
//! expanded to 8-bit RGBA.

use std::io::Cursor;

use png::{ColorType, Decoder, DecodingError, Transformations};

use crate::edit::EditError;
use crate::limits::{IMPORT_MAX_BYTES, IMPORT_MAX_SIDE};

/// The bytes of one pixel of grey and alpha.
const GREY_ALPHA_BYTES: usize = 2;
/// The bytes of one RGBA pixel.
pub(super) const RGBA_BYTES: usize = 4;

/// An image as 8-bit RGBA, 4 bytes per pixel, row by row.
pub(super) struct RgbaImage {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) pixels: Vec<u8>,
}

/// The first image of the PNG file `bytes`, as 8-bit RGBA.
pub(super) fn decode(bytes: &[u8]) -> Result<RgbaImage, EditError> {
    if bytes.len() > IMPORT_MAX_BYTES {
        return Err(EditError::ImageTooLarge);
    }
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(
        Transformations::EXPAND | Transformations::STRIP_16 | Transformations::ALPHA,
    );
    let header = decoder.read_header_info().map_err(malformed)?;
    if header.width > IMPORT_MAX_SIDE || header.height > IMPORT_MAX_SIDE {
        return Err(EditError::ImageTooLarge);
    }
    let mut reader = decoder.read_info().map_err(malformed)?;
    let mut buffer = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or(EditError::ImageTooLarge)?
    ];
    let frame = reader.next_frame(&mut buffer).map_err(malformed)?;
    buffer.truncate(frame.buffer_size());
    let pixels = to_rgba(buffer, frame.color_type)?;
    Ok(RgbaImage {
        width: frame.width,
        height: frame.height,
        pixels,
    })
}

fn malformed(error: DecodingError) -> EditError {
    match error {
        DecodingError::LimitsExceeded => EditError::ImageTooLarge,
        _ => EditError::ImageMalformed,
    }
}

/// The 8-bit output of the decoder — grey and alpha, or RGBA — as RGBA.
fn to_rgba(buffer: Vec<u8>, color_type: ColorType) -> Result<Vec<u8>, EditError> {
    match color_type {
        ColorType::Rgba => Ok(buffer),
        ColorType::GrayscaleAlpha => Ok(buffer
            .as_chunks::<GREY_ALPHA_BYTES>()
            .0
            .iter()
            .flat_map(|&[grey, alpha]| [grey, grey, grey, alpha])
            .collect()),
        _ => Err(EditError::ImageMalformed),
    }
}
