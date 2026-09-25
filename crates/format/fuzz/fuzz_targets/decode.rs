//! Any bytes: the decoder refuses them or accepts them, never panics, and a payload it accepts
//! never fails later.

#![no_main]

use libfuzzer_sys::fuzz_target;
use life_pixel_format::{Payload, apply_frame};

fuzz_target!(|bytes: &[u8]| {
    let Ok(payload) = Payload::parse(bytes) else {
        return;
    };
    let pixel_count = usize::from(payload.width()) * usize::from(payload.height());
    let mut indices = vec![0; pixel_count];
    let mut frame_count = 0;
    for frame in payload.frames() {
        let applied = apply_frame(&frame, &mut indices, payload.palette_len());
        assert_eq!(applied, Ok(()), "a parsed frame applies");
        assert!(
            indices
                .iter()
                .all(|&index| u16::from(index) < payload.palette_len())
        );
        frame_count += 1;
    }
    assert_eq!(
        frame_count,
        payload.frame_count(),
        "every frame is read back"
    );
    for index in 0..payload.tag_count() {
        assert!(payload.tag(index).is_some(), "every tag is read back");
    }
    assert!(payload.palette_entry(0).is_some());
});
