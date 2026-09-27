//! The README example, decoded field by field and frame by frame.

mod support;

use life_pixel_format::{FrameKind, LoopMode, Payload, Rgba, Tag, apply_frame};
use support::README_EXAMPLE;

const RED: Rgba = Rgba {
    r: 0xFF,
    g: 0,
    b: 0,
    a: 0xFF,
};

#[test]
fn decodes_the_header_palette_and_title() {
    let payload = Payload::parse(&README_EXAMPLE).unwrap();

    assert_eq!((payload.width(), payload.height()), (2, 2));
    assert_eq!(payload.frame_count(), 2);
    assert_eq!(payload.palette_len(), 2);
    assert_eq!(payload.palette_entry(0), Some(Rgba::default()));
    assert_eq!(payload.palette_entry(1), Some(RED));
    assert_eq!(payload.palette_entry(2), None);
    assert_eq!(payload.title(), "Hi");
}

#[test]
fn decodes_the_tags() {
    let payload = Payload::parse(&README_EXAMPLE).unwrap();

    assert_eq!(payload.tag_count(), 1);
    let idle = Tag {
        name: "idle",
        first: 0,
        last: 1,
        loop_mode: LoopMode::Loop,
    };
    assert_eq!(payload.tag(0), Some(idle));
    assert_eq!(payload.tag(1), None);
}

#[test]
fn applies_a_key_frame_then_a_delta_frame() {
    let payload = Payload::parse(&README_EXAMPLE).unwrap();
    let frames: Vec<_> = payload.frames().collect();
    let mut indices = [0xAA; 4];

    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].kind, FrameKind::Key);
    assert_eq!(frames[0].duration_ms, 100);
    apply_frame(&frames[0], &mut indices, payload.palette_len()).unwrap();
    assert_eq!(indices, [1, 1, 1, 1]);

    assert_eq!(frames[1].kind, FrameKind::Delta);
    apply_frame(&frames[1], &mut indices, payload.palette_len()).unwrap();
    assert_eq!(indices, [1, 1, 1, 0]);
}

#[cfg(feature = "encode")]
#[test]
fn the_encoder_writes_the_readme_example() {
    use life_pixel_format::encode;

    let animation = support::decode_animation(&README_EXAMPLE).unwrap();
    assert_eq!(encode(&animation).unwrap(), README_EXAMPLE);
}
