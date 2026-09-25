use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use life_pixel_format::{AnimationData, FrameData, Rgba, encode};

use super::*;

const TRANSPARENT: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};
const RED: Rgba = Rgba {
    r: 255,
    g: 0,
    b: 77,
    a: 255,
};
const YELLOW: Rgba = Rgba {
    r: 255,
    g: 236,
    b: 39,
    a: 128,
};

/// Four frames of 4 × 1 pixels: a key frame, two delta frames, then a key frame again.
const INDICES: [[u8; 4]; 4] = [[1, 1, 1, 1], [1, 1, 1, 2], [1, 1, 2, 2], [2, 2, 2, 2]];
const DURATIONS: [u16; 4] = [100, 50, 200, 80];

fn sample() -> Vec<u8> {
    let frames = INDICES.iter().zip(DURATIONS);
    encode(&AnimationData {
        width: 4,
        height: 1,
        palette: vec![TRANSPARENT, RED, YELLOW],
        title: String::new(),
        tags: Vec::new(),
        frames: frames
            .map(|(indices, duration_ms)| FrameData {
                duration_ms,
                indices: indices.to_vec(),
            })
            .collect(),
    })
    .unwrap()
}

fn loaded(payload: &[u8]) -> Frames {
    Frames::new(payload, &Payload::parse(payload).unwrap()).unwrap()
}

fn rgba_of(frame: usize) -> Vec<u8> {
    let palette = [TRANSPARENT, RED, YELLOW];
    let pixels = INDICES[frame]
        .iter()
        .map(|&index| palette[usize::from(index)]);
    pixels
        .flat_map(|Rgba { r, g, b, a }| [r, g, b, a])
        .collect()
}

#[test]
fn the_sample_mixes_key_and_delta_frames() {
    let payload = sample();
    let kinds: Vec<_> = Payload::parse(&payload)
        .unwrap()
        .frames()
        .map(|frame| frame.kind)
        .collect();

    assert_eq!(
        kinds,
        [
            FrameKind::Key,
            FrameKind::Delta,
            FrameKind::Delta,
            FrameKind::Key
        ]
    );
}

#[test]
fn nothing_is_painted_before_the_first_show() {
    let frames = loaded(&sample());

    assert_eq!(frames.framebuffer, [0; 16]);
}

#[test]
fn each_frame_shown_in_order_is_painted_through_the_palette() {
    let payload = sample();
    let mut frames = loaded(&payload);

    for frame in 0..4 {
        frames.show(&payload, u16::try_from(frame).unwrap());
        assert_eq!(frames.framebuffer, rgba_of(frame), "frame {frame}");
    }
}

#[test]
fn a_delta_frame_shown_first_is_decoded_from_the_key_frame_before_it() {
    let payload = sample();
    let mut frames = loaded(&payload);

    frames.show(&payload, 2);

    assert_eq!(frames.framebuffer, rgba_of(2));
}

#[test]
fn a_frame_before_the_one_shown_is_decoded_again_from_its_key_frame() {
    let payload = sample();
    let mut frames = loaded(&payload);
    frames.show(&payload, 3);

    frames.show(&payload, 1);

    assert_eq!(frames.framebuffer, rgba_of(1));
}

#[test]
fn a_frame_further_ahead_applies_the_deltas_in_between() {
    let payload = sample();
    let mut frames = loaded(&payload);
    frames.show(&payload, 0);

    frames.show(&payload, 2);

    assert_eq!(frames.framebuffer, rgba_of(2));
}

#[test]
fn durations_come_from_the_frame_table() {
    let frames = loaded(&sample());

    let durations = [0, 1, 2, 3].map(|frame| frames.duration_ms(frame));

    assert_eq!(durations, DURATIONS);
}

#[test]
fn the_framebuffer_never_moves() {
    let payload = sample();
    let mut frames = loaded(&payload);
    let address = frames.framebuffer_address();

    frames.show(&payload, 3);
    frames.show(&payload, 0);

    assert_eq!(frames.framebuffer_address(), address);
}
