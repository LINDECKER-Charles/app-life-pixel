//! APNG exports, decoded back: indexed colour with full alpha, delays and plays.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::io::Cursor;

use common::{BLINK, BLINK_FRAMES, MASCOT, MASCOT_COLOURS, MASCOT_FRAMES, animation, options};
use life_pixel_compiler::{ClassicOptions, export_apng};
use png::{ColorType, Decoder, Transformations};

struct Decoded {
    color_type: ColorType,
    size: (u32, u32),
    palette: Vec<u8>,
    alpha: Vec<u8>,
    num_plays: u32,
    /// Each frame's delay, as a numerator and a denominator of seconds.
    delays: Vec<(u16, u16)>,
    frames: Vec<Vec<u8>>,
}

fn decode(bytes: &[u8]) -> Decoded {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    let info = reader.info();
    let animation = info.animation_control.unwrap();
    let mut decoded = Decoded {
        color_type: info.color_type,
        size: (info.width, info.height),
        palette: info.palette.as_deref().unwrap().to_vec(),
        alpha: info.trns.as_deref().unwrap().to_vec(),
        num_plays: animation.num_plays,
        delays: Vec::new(),
        frames: Vec::new(),
    };
    for _ in 0..animation.num_frames {
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(frame.buffer_size());
        let control = reader.info().frame_control.unwrap();
        decoded.delays.push((control.delay_num, control.delay_den));
        decoded.frames.push(buffer);
    }
    decoded
}

#[test]
fn frames_are_indexed_with_the_palette_and_its_full_alpha() {
    let file = export_apng(&animation(MASCOT), &ClassicOptions::default()).unwrap();
    let decoded = decode(&file.bytes);
    assert_eq!(decoded.color_type, ColorType::Indexed);
    assert_eq!(decoded.size, (4, 3));
    let rgb: Vec<u8> = MASCOT_COLOURS
        .iter()
        .flat_map(|colour| &colour[..3])
        .copied()
        .collect();
    assert_eq!(decoded.palette, rgb);
    assert_eq!(decoded.alpha, [0, 255, 255, 128, 0]);
    assert_eq!(decoded.frames, MASCOT_FRAMES.map(|frame| frame.to_vec()));
}

#[test]
fn delays_are_milliseconds() {
    let decoded = decode(
        &export_apng(&animation(BLINK), &ClassicOptions::default())
            .unwrap()
            .bytes,
    );
    let delays = [10, 40, 1_000, 65_535, 100].map(|ms| (ms, 1_000));
    assert_eq!(decoded.delays, delays);
}

#[test]
fn a_looping_range_plays_forever_and_a_range_played_once_plays_once() {
    let mascot = animation(MASCOT);
    let every_frame = decode(
        &export_apng(&mascot, &ClassicOptions::default())
            .unwrap()
            .bytes,
    );
    assert_eq!(every_frame.num_plays, 0);
    let idle = decode(
        &export_apng(&mascot, &options(Some("idle"), 1))
            .unwrap()
            .bytes,
    );
    assert_eq!((idle.num_plays, idle.frames.len()), (0, 2));
    let jump = decode(
        &export_apng(&mascot, &options(Some("jump"), 1))
            .unwrap()
            .bytes,
    );
    assert_eq!((jump.num_plays, jump.frames.len()), (1, 1));
}

#[test]
fn a_tag_and_a_scale_export_its_frames_enlarged() {
    let file = export_apng(&animation(BLINK), &options(Some("blink"), 2)).unwrap();
    assert_eq!(file.name, "blink-2.apng");
    let decoded = decode(&file.bytes);
    assert_eq!(decoded.size, (4, 4));
    assert_eq!(decoded.num_plays, 1);
    assert_eq!(decoded.frames.len(), 3);
    let [top_left, top_right, bottom_left, bottom_right] = BLINK_FRAMES[1];
    let top = [top_left, top_left, top_right, top_right];
    let bottom = [bottom_left, bottom_left, bottom_right, bottom_right];
    assert_eq!(decoded.frames[0], [top, top, bottom, bottom].concat());
}
