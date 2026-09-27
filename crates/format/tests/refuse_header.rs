//! Payloads refused for their length or their header, one test per error.

mod support;

use life_pixel_format::bounds::{MAX_FRAMES, MAX_PAYLOAD_BYTES, MAX_SIDE};
use life_pixel_format::{DecodeError, Payload};
use support::{README_EXAMPLE, example_with, example_with_u16, offset};

fn refusal(payload: &[u8]) -> Option<DecodeError> {
    Payload::parse(payload).err()
}

#[test]
fn the_readme_example_is_accepted() {
    assert!(Payload::parse(&README_EXAMPLE).is_ok());
}

#[test]
fn a_payload_beyond_its_bound_is_refused_before_any_byte_is_read() {
    let payload = vec![0xFF; usize::try_from(MAX_PAYLOAD_BYTES).unwrap() + 1];
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn a_bad_magic_is_refused() {
    assert_eq!(
        refusal(&example_with(offset::MAGIC, b"LPIY")),
        Some(DecodeError::BadMagic)
    );
}

#[test]
fn an_unknown_format_version_is_refused() {
    for version in [0, 2, u16::MAX] {
        let payload = example_with_u16(offset::FORMAT_VERSION, version);
        assert_eq!(refusal(&payload), Some(DecodeError::UnknownFormatVersion));
    }
}

#[test]
fn an_unknown_abi_version_is_refused() {
    for version in [0, 2, u16::MAX] {
        let payload = example_with_u16(offset::ABI_VERSION, version);
        assert_eq!(refusal(&payload), Some(DecodeError::UnknownAbiVersion));
    }
}

#[test]
fn a_width_beyond_its_bound_is_refused() {
    let payload = example_with_u16(offset::WIDTH, MAX_SIDE + 1);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn a_height_beyond_its_bound_is_refused() {
    let payload = example_with_u16(offset::HEIGHT, MAX_SIDE + 1);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn a_frame_count_beyond_its_bound_is_refused() {
    let payload = example_with_u16(offset::FRAME_COUNT, MAX_FRAMES + 1);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn a_zero_width_is_refused() {
    assert_eq!(
        refusal(&example_with_u16(offset::WIDTH, 0)),
        Some(DecodeError::Malformed)
    );
}

#[test]
fn a_zero_height_is_refused() {
    assert_eq!(
        refusal(&example_with_u16(offset::HEIGHT, 0)),
        Some(DecodeError::Malformed)
    );
}

#[test]
fn a_zero_frame_count_is_refused() {
    assert_eq!(
        refusal(&example_with_u16(offset::FRAME_COUNT, 0)),
        Some(DecodeError::Malformed)
    );
}

#[test]
fn a_non_zero_reserved_field_is_refused() {
    assert_eq!(
        refusal(&example_with_u16(offset::RESERVED, 1)),
        Some(DecodeError::Malformed)
    );
}

#[test]
fn a_truncated_header_is_refused() {
    for len in offset::HEADER {
        assert_eq!(
            refusal(&README_EXAMPLE[..len]),
            Some(DecodeError::Malformed),
            "{len} bytes"
        );
    }
}

#[test]
fn a_trailing_byte_is_refused() {
    let mut payload = README_EXAMPLE.to_vec();
    payload.push(0);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}
