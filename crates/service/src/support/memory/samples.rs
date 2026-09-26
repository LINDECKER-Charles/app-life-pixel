//! Uploads for the screenshot tests: a PNG carrying metadata, a JPEG, and a PNG whose header lies
//! about its size.

use std::io::Cursor;

use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;
use png::{BitDepth, ColorType, Encoder};

/// The eight bytes every PNG starts with.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
/// An RGBA pixel of the samples.
const PIXEL: [u8; 4] = [200, 40, 90, 255];
/// A PNG's bit depth, colour type (RGBA), compression, filter and interlace, after its sides.
const IHDR_TAIL: [u8; 5] = [8, 6, 0, 0, 0];
/// The reflected polynomial of the CRC-32 PNG chunks carry.
const CRC32_POLYNOMIAL: u32 = 0xEDB8_8320;
/// The JPEG samples' quality.
const JPEG_QUALITY: u8 = 90;

/// A `width × height` PNG whose `tEXt` chunk holds `key: value`.
///
/// # Panics
///
/// When the encoder fails, which a valid size never makes it.
#[must_use]
#[allow(clippy::expect_used)] // A test helper: a failure is the test's.
pub fn png_screenshot_with_text(
    (width, height): (u32, u32),
    (key, value): (&str, &str),
) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder
        .add_text_chunk(key.to_owned(), value.to_owned())
        .expect("a text chunk");
    let mut writer = encoder.write_header().expect("a header");
    writer
        .write_image_data(&pixels(width, height, &PIXEL))
        .expect("the pixels");
    writer.finish().expect("the end");
    bytes
}

/// A `width × height` JPEG.
///
/// # Panics
///
/// When the encoder fails, which a valid size never makes it.
#[must_use]
#[allow(clippy::expect_used)] // A test helper: a failure is the test's.
pub fn jpeg_screenshot((width, height): (u32, u32)) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    let rgb = pixels(width, height, &PIXEL[..3]);
    JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY)
        .encode(&rgb, width, height, ExtendedColorType::Rgb8)
        .expect("a JPEG");
    bytes.into_inner()
}

/// A PNG whose header claims `side × side` RGBA pixels, followed by an empty `IDAT`: a decoder
/// that believed it would allocate `side² × 4` bytes before finding no pixel.
#[must_use]
pub fn png_claiming_side(side: u32) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(&side.to_be_bytes());
    header.extend_from_slice(&side.to_be_bytes());
    header.extend_from_slice(&IHDR_TAIL);
    let mut bytes = PNG_SIGNATURE.to_vec();
    for (kind, data) in [(b"IHDR", header.as_slice()), (b"IDAT", &[]), (b"IEND", &[])] {
        push_chunk(&mut bytes, *kind, data);
    }
    bytes
}

/// `width × height` copies of `pixel`.
fn pixels(width: u32, height: u32, pixel: &[u8]) -> Vec<u8> {
    let count = usize::try_from(u64::from(width) * u64::from(height)).unwrap_or(0);
    pixel.repeat(count)
}

/// Appends the PNG chunk `kind` holding `data` to `bytes`, with its length and CRC.
fn push_chunk(bytes: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    let length = u32::try_from(data.len()).unwrap_or(u32::MAX);
    bytes.extend_from_slice(&length.to_be_bytes());
    let start = bytes.len();
    bytes.extend_from_slice(&kind);
    bytes.extend_from_slice(data);
    let crc = crc32(&bytes[start..]);
    bytes.extend_from_slice(&crc.to_be_bytes());
}

/// The CRC-32 of `bytes`, bit by bit: the samples are a few bytes long.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (CRC32_POLYNOMIAL & mask);
        }
    }
    !crc
}
