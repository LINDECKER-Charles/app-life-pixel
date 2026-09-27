//! Golden images: RGBA renders compared pixel for pixel with the PNG files of
//! `crates/core/tests/golden/`. `LP_UPDATE_GOLDEN=1` rewrites them instead, for review in the diff.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use png::{BitDepth, ColorType, Decoder, Encoder};

/// The variable that rewrites the golden images instead of comparing with them.
const UPDATE_VARIABLE: &str = "LP_UPDATE_GOLDEN";
/// The bytes of one RGBA pixel.
const RGBA_BYTES: usize = 4;

/// An RGBA image: 4 bytes per pixel, row by row.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    /// The images side by side, left to right; all have the height of the first.
    pub fn side_by_side(images: &[Image]) -> Self {
        let height = images[0].height;
        let width = images.iter().map(|image| image.width).sum();
        let mut pixels = Vec::new();
        for row in 0..height {
            for image in images {
                let row_bytes = image.width as usize * RGBA_BYTES;
                let start = row as usize * row_bytes;
                pixels.extend_from_slice(&image.pixels[start..start + row_bytes]);
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }
}

/// Compares `image` with the golden image `name`, or rewrites it under `LP_UPDATE_GOLDEN=1`.
pub fn assert_golden(name: &str, image: &Image) {
    let path = golden_path(name);
    if std::env::var(UPDATE_VARIABLE).is_ok_and(|value| value == "1") {
        write(&path, image);
        return;
    }
    let expected = read(&path);
    assert_eq!(
        (expected.width, expected.height),
        (image.width, image.height),
        "{name}: size differs from the golden image; rerun with {UPDATE_VARIABLE}=1 to review"
    );
    let first_difference = expected
        .pixels
        .chunks(RGBA_BYTES)
        .zip(image.pixels.chunks(RGBA_BYTES))
        .position(|(expected, actual)| expected != actual);
    if let Some(pixel) = first_difference {
        let (x, y) = (pixel % image.width as usize, pixel / image.width as usize);
        panic!("{name}: pixel ({x}, {y}) differs from the golden image");
    }
}

fn golden_path(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    directory.join(format!("{name}.png"))
}

fn read(path: &PathBuf) -> Image {
    let file = File::open(path).unwrap_or_else(|error| {
        panic!(
            "{}: {error}; run with {UPDATE_VARIABLE}=1 to create it",
            path.display()
        )
    });
    let mut reader = Decoder::new(BufReader::new(file)).read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(frame.color_type, ColorType::Rgba, "{}", path.display());
    pixels.truncate(frame.buffer_size());
    Image {
        width: frame.width,
        height: frame.height,
        pixels,
    }
}

fn write(path: &PathBuf, image: &Image) {
    let file = File::create(path).unwrap();
    let mut encoder = Encoder::new(file, image.width, image.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.pixels).unwrap();
    writer.finish().unwrap();
}
