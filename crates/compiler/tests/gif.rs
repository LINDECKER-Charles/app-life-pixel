//! GIF exports, decoded back: palette, transparency, disposal, delays and looping.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::{BLINK, BLINK_FRAMES, MASCOT, MASCOT_FRAMES, animation, options};
use gif::{ColorOutput, DecodeOptions, DisposalMethod, Repeat};
use life_pixel_compiler::{ClassicOptions, export_gif};

/// NETSCAPE2.0's application identifier, which starts the loop extension.
const LOOP_EXTENSION: &[u8] = b"NETSCAPE2.0";

struct DecodedFrame {
    delay: u16,
    dispose: DisposalMethod,
    transparent: Option<u8>,
    size: (u16, u16),
    indices: Vec<u8>,
}

struct Decoded {
    palette: Vec<u8>,
    repeat: Repeat,
    frames: Vec<DecodedFrame>,
}

fn decode(bytes: &[u8]) -> Decoded {
    let mut options = DecodeOptions::new();
    options.set_color_output(ColorOutput::Indexed);
    let mut decoder = options.read_info(bytes).unwrap();
    let mut frames = Vec::new();
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        frames.push(DecodedFrame {
            delay: frame.delay,
            dispose: frame.dispose,
            transparent: frame.transparent,
            size: (frame.width, frame.height),
            indices: frame.buffer.to_vec(),
        });
    }
    Decoded {
        palette: decoder.global_palette().unwrap().to_vec(),
        repeat: decoder.repeat(),
        frames,
    }
}

fn has_loop_extension(bytes: &[u8]) -> bool {
    bytes
        .windows(LOOP_EXTENSION.len())
        .any(|window| window == LOOP_EXTENSION)
}

#[test]
fn every_frame_is_full_size_transparent_at_0_and_restored_to_background() {
    let file = export_gif(&animation(MASCOT), &ClassicOptions::default()).unwrap();
    let decoded = decode(&file.bytes);
    assert_eq!(decoded.frames.len(), 3);
    for frame in &decoded.frames {
        assert_eq!(frame.size, (4, 3));
        assert_eq!(frame.transparent, Some(0));
        assert_eq!(frame.dispose, DisposalMethod::Background);
    }
}

#[test]
fn the_palette_drops_alpha_and_fully_transparent_colours_show_as_index_0() {
    let file = export_gif(&animation(MASCOT), &ClassicOptions::default()).unwrap();
    let decoded = decode(&file.bytes);
    let colours = [0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255];
    assert_eq!(decoded.palette[..15], colours);
    assert_eq!(decoded.palette.len(), 8 * 3, "padded to a power of 2");
    assert_eq!(decoded.frames[0].indices, MASCOT_FRAMES[0]);
    // Colour 4 has no alpha at all: it becomes the transparent index; colour 3 stays opaque.
    assert_eq!(
        decoded.frames[1].indices,
        [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0]
    );
    assert_eq!(
        decoded.frames[2].indices,
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn delays_are_hundredths_rounded_and_at_least_2() {
    let mascot = decode(
        &export_gif(&animation(MASCOT), &ClassicOptions::default())
            .unwrap()
            .bytes,
    );
    let delays: Vec<u16> = mascot.frames.iter().map(|frame| frame.delay).collect();
    assert_eq!(delays, [10, 15, 3]);
    let blink = decode(
        &export_gif(&animation(BLINK), &ClassicOptions::default())
            .unwrap()
            .bytes,
    );
    let delays: Vec<u16> = blink.frames.iter().map(|frame| frame.delay).collect();
    assert_eq!(delays, [2, 4, 100, 6_554, 10]);
}

#[test]
fn a_looping_range_loops_forever_and_a_range_played_once_has_no_loop_extension() {
    let mascot = animation(MASCOT);
    let looping = export_gif(&mascot, &options(Some("idle"), 1)).unwrap();
    assert_eq!(decode(&looping.bytes).repeat, Repeat::Infinite);
    assert!(has_loop_extension(&looping.bytes));
    let once = export_gif(&mascot, &options(Some("jump"), 1)).unwrap();
    assert!(!has_loop_extension(&once.bytes));
    assert_eq!(decode(&once.bytes).frames.len(), 1);
}

#[test]
fn a_tag_exports_its_frames_only() {
    let file = export_gif(&animation(BLINK), &options(Some("blink"), 1)).unwrap();
    let decoded = decode(&file.bytes);
    let indices: Vec<Vec<u8>> = decoded
        .frames
        .iter()
        .map(|frame| frame.indices.clone())
        .collect();
    let expected: Vec<Vec<u8>> = BLINK_FRAMES[1..=3]
        .iter()
        .map(|frame| frame.to_vec())
        .collect();
    assert_eq!(indices, expected);
    assert_eq!(file.name, "blink-2.gif");
}

#[test]
fn a_scale_repeats_each_pixel() {
    let file = export_gif(&animation(BLINK), &options(None, 3)).unwrap();
    let decoded = decode(&file.bytes);
    assert_eq!(decoded.frames[0].size, (6, 6));
    let row_1 = [1, 1, 1, 2, 2, 2];
    let row_2 = [2, 2, 2, 1, 1, 1];
    let expected = [row_1, row_1, row_1, row_2, row_2, row_2].concat();
    assert_eq!(decoded.frames[0].indices, expected);
}
