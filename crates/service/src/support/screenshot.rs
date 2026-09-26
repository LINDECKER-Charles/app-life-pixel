//! A support screenshot: a PNG or a JPEG, checked against the limits before it is decoded,
//! decoded under them, and re-encoded as PNG so that nothing of the upload but its pixels is
//! kept — metadata, colour profile and trailing bytes included.

use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, ImageError, ImageFormat, ImageReader, Limits};
use life_pixel_core::limits::{SCREENSHOT_MAX_BYTES, SCREENSHOT_MAX_SIDE};
use thiserror::Error;

/// The most bytes a pixel takes once decoded: 16-bit RGBA.
const MAX_BYTES_PER_PIXEL: u64 = 8;
/// What a decoder may allocate beyond the pixels themselves, as a multiple of them.
const DECODER_ALLOCATION_FACTOR: u64 = 2;

/// Why an upload is not a screenshot: for the logs and the tests, the person only learns
/// `support.screenshot`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum ScreenshotRejection {
    /// More than `SCREENSHOT_MAX_BYTES`.
    #[error("too many bytes")]
    TooManyBytes,
    /// Neither a PNG nor a JPEG.
    #[error("not a PNG or a JPEG")]
    Format,
    /// Its header claims a side above `SCREENSHOT_MAX_SIDE`, or none.
    #[error("dimensions out of bounds")]
    Dimensions,
    /// It does not decode.
    #[error("undecodable")]
    Undecodable,
    /// Its pixels could not be encoded again.
    #[error("cannot be re-encoded")]
    Encoding,
}

/// A screenshot's pixels, encoded as PNG.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screenshot(Bytes);

impl Screenshot {
    /// The screenshot of `upload`: its size, type and dimensions checked before it is decoded,
    /// then decoded under the limits and re-encoded as PNG. Decoding takes time: call it on a
    /// blocking thread.
    ///
    /// # Errors
    ///
    /// The first check `upload` fails.
    pub fn prepare(upload: &[u8]) -> Result<Self, ScreenshotRejection> {
        if upload.len() > SCREENSHOT_MAX_BYTES {
            return Err(ScreenshotRejection::TooManyBytes);
        }
        let format = image::guess_format(upload).map_err(|_| ScreenshotRejection::Format)?;
        if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
            return Err(ScreenshotRejection::Format);
        }
        check_dimensions(upload, format)?;
        let decoded = reader(upload, format).decode().map_err(rejection)?;
        encode(&decoded).map(Self)
    }

    /// The PNG.
    #[must_use]
    pub fn into_png(self) -> Bytes {
        self.0
    }
}

/// A reader of `upload` as `format`, under the screenshots' limits.
fn reader(upload: &[u8], format: ImageFormat) -> ImageReader<Cursor<&[u8]>> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(SCREENSHOT_MAX_SIDE);
    limits.max_image_height = Some(SCREENSHOT_MAX_SIDE);
    let pixels = u64::from(SCREENSHOT_MAX_SIDE).pow(2);
    limits.max_alloc = Some(pixels * MAX_BYTES_PER_PIXEL * DECODER_ALLOCATION_FACTOR);
    let mut reader = ImageReader::with_format(Cursor::new(upload), format);
    reader.limits(limits);
    reader
}

/// Reads the dimensions `upload`'s header claims, without decoding its pixels.
fn check_dimensions(upload: &[u8], format: ImageFormat) -> Result<(), ScreenshotRejection> {
    let (width, height) = reader(upload, format)
        .into_dimensions()
        .map_err(rejection)?;
    let sides = 1..=SCREENSHOT_MAX_SIDE;
    if !sides.contains(&width) || !sides.contains(&height) {
        return Err(ScreenshotRejection::Dimensions);
    }
    Ok(())
}

/// The rejection of a decoder's `error`: its limits reached, or anything else.
fn rejection(error: ImageError) -> ScreenshotRejection {
    match error {
        ImageError::Limits(_) => ScreenshotRejection::Dimensions,
        _ => ScreenshotRejection::Undecodable,
    }
}

/// The pixels of `decoded` as a PNG, without any metadata.
fn encode(decoded: &DynamicImage) -> Result<Bytes, ScreenshotRejection> {
    let mut png = Cursor::new(Vec::new());
    decoded
        .write_to(&mut png, ImageFormat::Png)
        .map_err(|_| ScreenshotRejection::Encoding)?;
    Ok(Bytes::from(png.into_inner()))
}
