//! Payloads refused for their frames or their operations, one test per error.

mod support;

use life_pixel_format::{DecodeError, Frame, FrameKind, Payload, apply_frame};
use support::{README_EXAMPLE, example_with, example_with_u16, offset, payload_with_frames};

const KEY: u8 = 0;
const DELTA: u8 = 1;

fn refusal(payload: &[u8]) -> Option<DecodeError> {
    Payload::parse(payload).err()
}

#[test]
fn truncated_frames_are_refused() {
    for len in offset::FRAMES {
        assert_eq!(
            refusal(&README_EXAMPLE[..len]),
            Some(DecodeError::Malformed),
            "{len} bytes"
        );
    }
}

#[test]
fn a_zero_duration_is_refused() {
    let payload = example_with_u16(offset::FRAME_0_DURATION, 0);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_frame_kind_above_1_is_refused() {
    let payload = example_with(offset::FRAME_1_KIND, &[2]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_delta_frame_0_is_refused() {
    let payload = example_with(offset::FRAME_0_KIND, &[DELTA]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn operation_11_is_refused() {
    let payload = example_with(offset::FRAME_0_DATA, &[0xC3, 0x01]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_skip_in_a_key_frame_is_refused() {
    let skip_then_run: &[u8] = &[0x01, 0x41, 0x01];
    let in_delta = payload_with_frames(2, 2, &[(KEY, &[0x43, 0x01]), (DELTA, skip_then_run)]);
    let in_key = payload_with_frames(2, 2, &[(KEY, &[0x43, 0x01]), (KEY, skip_then_run)]);

    assert!(Payload::parse(&in_delta).is_ok());
    assert_eq!(refusal(&in_key), Some(DecodeError::Malformed));
}

#[test]
fn a_varint_of_6_bytes_is_refused() {
    // RUN, n = 63, a varint of value 0 — a count of 64 — then index 1.
    let five_bytes = payload_with_frames(64, 1, &[(KEY, &[0x7F, 0x80, 0x80, 0x80, 0x80, 0x00, 1])]);
    let six_bytes = payload_with_frames(
        64,
        1,
        &[(KEY, &[0x7F, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00, 1])],
    );

    assert!(Payload::parse(&five_bytes).is_ok());
    assert_eq!(refusal(&six_bytes), Some(DecodeError::Malformed));
}

#[test]
fn a_run_index_at_the_palette_count_is_refused() {
    let payload = example_with(offset::FRAME_0_DATA + 1, &[2]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_literal_index_at_the_palette_count_is_refused() {
    let payload = example_with(offset::FRAME_1_DATA + 2, &[2]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn counts_that_fall_short_of_the_canvas_are_refused() {
    let payload = example_with(offset::FRAME_0_DATA, &[0x42, 0x01]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn counts_that_pass_the_canvas_are_refused() {
    let payload = example_with(offset::FRAME_0_DATA, &[0x44, 0x01]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_huge_varint_count_is_refused() {
    let payload = payload_with_frames(2, 2, &[(KEY, &[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 1])]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn an_operation_cut_by_the_end_of_the_frame_data_is_refused() {
    let run_without_index = payload_with_frames(2, 2, &[(KEY, &[0x43])]);
    let short_literal = payload_with_frames(2, 2, &[(KEY, &[0x83, 1, 1, 1])]);

    assert_eq!(refusal(&run_without_index), Some(DecodeError::Malformed));
    assert_eq!(refusal(&short_literal), Some(DecodeError::Malformed));
}

#[test]
fn apply_frame_refuses_a_skip_in_a_key_frame() {
    let frame = Frame {
        duration_ms: 100,
        kind: FrameKind::Key,
        data: &[0x03],
    };
    assert_eq!(
        apply_frame(&frame, &mut [0; 4], 2),
        Err(DecodeError::Malformed)
    );
}

#[test]
fn apply_frame_refuses_a_canvas_of_another_size() {
    let frame = Frame {
        duration_ms: 100,
        kind: FrameKind::Key,
        data: &[0x43, 0x01],
    };
    assert_eq!(
        apply_frame(&frame, &mut [0; 3], 2),
        Err(DecodeError::Malformed)
    );
    assert_eq!(
        apply_frame(&frame, &mut [0; 5], 2),
        Err(DecodeError::Malformed)
    );
}
