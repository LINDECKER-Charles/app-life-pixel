//! The animations the integration tests export, and their expected pixels.

#![allow(dead_code)] // Each test file uses its own share of the helpers.

use life_pixel_compiler::ClassicOptions;
use life_pixel_core::Animation;
use life_pixel_core::serialize::read_document;

/// 4 × 3, 3 frames of 100, 150 and 25 ms; a hidden layer; colour 3 half transparent, colour 4
/// fully transparent; tags `idle` (0–1, loop) and `jump` (2, once).
pub const MASCOT: &str = include_str!("../fixtures/mascot.json");
/// 2 × 2, titled `Blink — 2!`, 5 frames of 10, 40, 1000, 65535 and 100 ms, the third blank;
/// tag `blink` (1–3, once).
pub const BLINK: &str = include_str!("../fixtures/blink.json");

/// The composited palette indices of each mascot frame, row by row.
pub const MASCOT_FRAMES: [[u8; 12]; 3] = [
    [1, 1, 1, 1, 1, 3, 2, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1, 4, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4],
];
/// The composited palette indices of each blink frame, row by row.
pub const BLINK_FRAMES: [[u8; 4]; 5] = [[1, 2, 2, 1], [1, 0, 0, 1], [0; 4], [2; 4], [2, 0, 0, 0]];

/// The mascot's colours, red, green, blue and alpha.
pub const MASCOT_COLOURS: [[u8; 4]; 5] = [
    [0, 0, 0, 0],
    [0, 0, 0, 255],
    [255, 255, 255, 255],
    [255, 0, 0, 128],
    [0, 0, 255, 0],
];

/// The animation of a fixture document.
pub fn animation(document: &str) -> Animation {
    read_document(document.as_bytes()).unwrap()
}

/// Options for `tag`, at `scale`.
pub fn options(tag: Option<&str>, scale: u8) -> ClassicOptions {
    ClassicOptions {
        tag: tag.map(str::to_owned),
        scale,
    }
}
