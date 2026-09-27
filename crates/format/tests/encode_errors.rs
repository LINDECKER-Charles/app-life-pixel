//! Animations the encoder refuses, one test per error: what the decoder calls beyond a bound is
//! `BeyondBound`, every other broken rule `Inconsistent`.

#![cfg(feature = "encode")]

mod support;

use life_pixel_format::bounds::{
    MAX_FRAMES, MAX_SIDE, MAX_TAG_NAME_BYTES, MAX_TAGS, MAX_TITLE_BYTES,
};
use life_pixel_format::{AnimationData, EncodeError, Rgba, encode};
use support::{animation, palette, tag};

/// A valid 2 × 1 animation of 2 frames over 3 entries.
fn sound() -> AnimationData {
    animation(2, 1, vec![vec![0, 1], vec![2, 1]])
}

fn refusal(animation: &AnimationData) -> Option<EncodeError> {
    encode(animation).err()
}

#[test]
fn a_sound_animation_is_encoded() {
    assert!(encode(&sound()).is_ok());
}

#[test]
fn a_zero_side_is_inconsistent() {
    let mut zero_width = animation(0, 1, vec![vec![]]);
    assert_eq!(refusal(&zero_width), Some(EncodeError::Inconsistent));
    zero_width.width = 1;
    zero_width.height = 0;
    assert_eq!(refusal(&zero_width), Some(EncodeError::Inconsistent));
}

#[test]
fn a_side_beyond_its_bound_is_refused() {
    let wide = animation(MAX_SIDE + 1, 1, vec![vec![0; usize::from(MAX_SIDE) + 1]]);
    let tall = animation(1, MAX_SIDE + 1, vec![vec![0; usize::from(MAX_SIDE) + 1]]);
    assert_eq!(refusal(&wide), Some(EncodeError::BeyondBound));
    assert_eq!(refusal(&tall), Some(EncodeError::BeyondBound));
}

#[test]
fn no_frame_is_inconsistent() {
    assert_eq!(
        refusal(&animation(1, 1, vec![])),
        Some(EncodeError::Inconsistent)
    );
}

#[test]
fn a_frame_count_beyond_its_bound_is_refused() {
    let frames = vec![vec![0]; usize::from(MAX_FRAMES) + 1];
    assert_eq!(
        refusal(&animation(1, 1, frames)),
        Some(EncodeError::BeyondBound)
    );
}

#[test]
fn an_empty_palette_is_inconsistent() {
    let mut animation = sound();
    animation.palette.clear();
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_palette_of_more_than_256_entries_is_inconsistent() {
    let mut animation = sound();
    animation.palette = palette(257);
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn an_opaque_entry_0_is_inconsistent() {
    let mut animation = sound();
    animation.palette[0] = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 1,
    };
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_title_beyond_its_bound_is_refused() {
    let mut animation = sound();
    animation.title = "t".repeat(usize::from(MAX_TITLE_BYTES) + 1);
    assert_eq!(refusal(&animation), Some(EncodeError::BeyondBound));
}

#[test]
fn a_tag_count_beyond_its_bound_is_refused() {
    let mut animation = sound();
    animation.tags = vec![tag("all", 0, 1); usize::from(MAX_TAGS) + 1];
    assert_eq!(refusal(&animation), Some(EncodeError::BeyondBound));
}

#[test]
fn a_tag_name_beyond_its_bound_is_refused() {
    let mut animation = sound();
    let name = "n".repeat(usize::from(MAX_TAG_NAME_BYTES) + 1);
    animation.tags = vec![tag(&name, 0, 1)];
    assert_eq!(refusal(&animation), Some(EncodeError::BeyondBound));
}

#[test]
fn an_empty_tag_name_is_inconsistent() {
    let mut animation = sound();
    animation.tags = vec![tag("", 0, 1)];
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_tag_out_of_range_is_inconsistent() {
    let mut animation = sound();
    animation.tags = vec![tag("reversed", 1, 0)];
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
    animation.tags = vec![tag("past the end", 0, 2)];
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_frame_of_the_wrong_size_is_inconsistent() {
    let mut animation = sound();
    animation.frames[1].indices.push(0);
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn an_index_at_the_palette_count_is_inconsistent() {
    let mut animation = sound();
    animation.frames[1].indices[0] = 3;
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_zero_duration_is_inconsistent() {
    let mut animation = sound();
    animation.frames[1].duration_ms = 0;
    assert_eq!(refusal(&animation), Some(EncodeError::Inconsistent));
}

#[test]
fn a_payload_beyond_its_bound_is_refused() {
    // Every frame a full canvas of alternating indices, each the reverse of the one before:
    // key frames of one LITERAL each, 4 MiB apiece, 17 of them passing 64 MiB.
    let pixel_count = usize::from(MAX_SIDE) * usize::from(MAX_SIDE);
    let even: Vec<u8> = (0..pixel_count)
        .map(|pixel| u8::from(pixel % 2 == 0))
        .collect();
    let odd: Vec<u8> = even.iter().map(|&index| 1 - index).collect();
    let frames = (0..17)
        .map(|frame| {
            if frame % 2 == 0 {
                even.clone()
            } else {
                odd.clone()
            }
        })
        .collect();
    let animation = animation(MAX_SIDE, MAX_SIDE, frames);
    assert_eq!(refusal(&animation), Some(EncodeError::BeyondBound));
}
