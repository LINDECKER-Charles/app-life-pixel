//! Generated animations encoded, then decoded back to the same animation.

#![cfg(feature = "encode")]

mod support;

use life_pixel_format::bounds::{
    MAX_FRAMES, MAX_SIDE, MAX_TAG_NAME_BYTES, MAX_TAGS, MAX_TITLE_BYTES,
};
use life_pixel_format::{AnimationData, FrameKind, Payload, encode};
use support::{animation, decode_animation, frame_kinds, pattern, tag};

/// Encodes `animation` twice, checks that both give the same bytes and that they decode back
/// to `animation`, and returns them.
#[track_caller]
fn assert_round_trip(animation: &AnimationData) -> Vec<u8> {
    let (Ok(bytes), Ok(again)) = (encode(animation), encode(animation)) else {
        panic!("a sound animation is refused");
    };
    assert_eq!(again, bytes, "the encoder is deterministic");
    assert_eq!(decode_animation(&bytes).as_ref(), Ok(animation));
    bytes
}

/// The data of frame `position` of a payload.
fn frame_data(bytes: &[u8], position: usize) -> Option<Vec<u8>> {
    let payload = Payload::parse(bytes).ok()?;
    Some(payload.frames().nth(position)?.data.to_vec())
}

#[test]
fn one_pixel() {
    assert_round_trip(&animation(1, 1, vec![vec![1], vec![0], vec![1]]));
}

#[test]
fn full_canvas() {
    let pixel_count = usize::from(MAX_SIDE) * usize::from(MAX_SIDE);
    let first = pattern(pixel_count, 16, 1);
    let mut second = first.clone();
    second[1_000..2_000].fill(3);
    let bytes = assert_round_trip(&animation(MAX_SIDE, MAX_SIDE, vec![first, second]));
    assert_eq!(
        frame_kinds(&bytes).unwrap(),
        [FrameKind::Key, FrameKind::Delta]
    );
}

#[test]
fn every_one_of_256_colours() {
    let indices = (0..=255).collect();
    assert_round_trip(&animation(16, 16, vec![indices]));
}

#[test]
fn the_maximum_frame_count() {
    let frames = (0..u32::from(MAX_FRAMES))
        .map(|seed| pattern(9, 4, seed))
        .collect();
    let bytes = assert_round_trip(&animation(3, 3, frames));
    assert_eq!(Payload::parse(&bytes).unwrap().frame_count(), MAX_FRAMES);
}

#[test]
fn the_longest_title_and_the_most_tags_with_the_longest_names() {
    let mut animation = animation(2, 1, vec![vec![0, 1], vec![1, 0]]);
    animation.title = "é".repeat(usize::from(MAX_TITLE_BYTES) / 2);
    let name = "n".repeat(usize::from(MAX_TAG_NAME_BYTES));
    animation.tags = (0..MAX_TAGS)
        .map(|index| tag(&name, index % 2, 1))
        .collect();
    assert_round_trip(&animation);
}

#[test]
fn long_runs_take_the_varint() {
    let bytes = assert_round_trip(&animation(300, 1, vec![vec![5; 300]]));
    // RUN, n = 63, varint 236 — 64 + 236 = 300 — then index 5.
    assert_eq!(frame_data(&bytes, 0).unwrap(), [0x7F, 0xEC, 0x01, 0x05]);
}

#[test]
fn long_literals_and_skips_take_the_varint() {
    let alternating: Vec<u8> = (0..200).map(|pixel| u8::from(pixel % 2 == 0)).collect();
    let mut changed_end = alternating.clone();
    changed_end[199] = 2;
    let bytes = assert_round_trip(&animation(200, 1, vec![alternating, changed_end]));
    // LITERAL, n = 63, varint 136 — 64 + 136 = 200.
    assert_eq!(frame_data(&bytes, 0).unwrap()[..3], [0xBF, 0x88, 0x01]);
    // SKIP, n = 63, varint 135 — 64 + 135 = 199 —, then a LITERAL of index 2.
    assert_eq!(
        frame_data(&bytes, 1).unwrap(),
        [0x3F, 0x87, 0x01, 0x80, 0x02]
    );
}

#[test]
fn frames_identical_to_their_predecessor_become_one_skip() {
    let indices = pattern(64, 4, 7);
    let bytes = assert_round_trip(&animation(8, 8, vec![indices; 3]));
    let kinds = [FrameKind::Key, FrameKind::Delta, FrameKind::Delta];
    assert_eq!(frame_kinds(&bytes).unwrap(), kinds);
    // SKIP, n = 63, varint 0: the 64 pixels.
    assert_eq!(frame_data(&bytes, 1).unwrap(), [0x3F, 0x00]);
}

#[test]
fn frame_0_and_the_first_frame_of_each_tag_are_key_frames() {
    let indices = pattern(16, 4, 3);
    let mut animation = animation(4, 4, vec![indices; 4]);
    animation.tags = vec![tag("walk", 2, 3)];
    let bytes = assert_round_trip(&animation);
    let kinds = [
        FrameKind::Key,
        FrameKind::Delta,
        FrameKind::Key,
        FrameKind::Delta,
    ];
    assert_eq!(frame_kinds(&bytes).unwrap(), kinds);
}

#[test]
fn a_delta_frame_is_kept_only_when_smaller_than_its_key_frame() {
    // Frame 1 changes every pixel: its delta encoding is as long as its key encoding.
    // Frame 2 changes one pixel: its delta encoding is shorter.
    let frames = vec![vec![0, 1, 2, 3], vec![1, 2, 3, 0], vec![1, 2, 3, 3]];
    let bytes = assert_round_trip(&animation(4, 1, frames));
    let kinds = [FrameKind::Key, FrameKind::Key, FrameKind::Delta];
    assert_eq!(frame_kinds(&bytes).unwrap(), kinds);
}
