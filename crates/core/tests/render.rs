//! Compositing a frame, and its RGBA through the palette.

mod common;

use common::{read, sample_document};
use life_pixel_core::render::{composite, composite_with, rgba, rgba_with};
use life_pixel_core::{Cel, FrameId, LayerId};
use serde_json::json;

/// Layer 1 is `[1, 1, 2, 0]`, layer 2 `[2, 0, 1, 0]` on frame 3; frame 4 is blank.
const FRAME: FrameId = FrameId::new(3);
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

#[test]
fn upper_layers_cover_lower_ones_except_where_they_are_0() {
    let animation = read(&sample_document()).unwrap();
    assert_eq!(composite(&animation, FRAME), [2, 1, 1, 0]);
    assert_eq!(composite(&animation, FrameId::new(4)), [0, 0, 0, 0]);
}

#[test]
fn hidden_layers_are_skipped() {
    let mut document = sample_document();
    document["layers"][1]["visible"] = json!(false);
    let animation = read(&document).unwrap();
    assert_eq!(composite(&animation, FRAME), [1, 1, 2, 0]);
    document["layers"][0]["visible"] = json!(false);
    assert_eq!(composite(&read(&document).unwrap(), FRAME), [0, 0, 0, 0]);
}

#[test]
fn a_replacement_cel_stands_in_for_its_layer() {
    let animation = read(&sample_document()).unwrap();
    let preview = Cel::new(vec![0, 0, 0, 2]);
    let replacement = (LayerId::new(2), &preview);
    assert_eq!(composite_with(&animation, FRAME, replacement), [1, 1, 2, 2]);
    let blank_frame = composite_with(&animation, FrameId::new(4), replacement);
    assert_eq!(blank_frame, [0, 0, 0, 2]);
}

#[test]
fn rgba_goes_through_the_palette_4_bytes_per_pixel() {
    let animation = read(&sample_document()).unwrap();
    assert_eq!(
        rgba(&animation, FRAME),
        [WHITE, BLACK, BLACK, TRANSPARENT].concat()
    );
    let preview = Cel::new(vec![0, 0, 0, 1]);
    let with_preview = rgba_with(&animation, FRAME, (LayerId::new(2), &preview));
    assert_eq!(with_preview, [BLACK, BLACK, WHITE, BLACK].concat());
}
