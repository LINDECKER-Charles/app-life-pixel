//! Payloads refused for their palette, title or tags, one test per error.

mod support;

use life_pixel_format::bounds::{MAX_TAG_NAME_BYTES, MAX_TAGS, MAX_TITLE_BYTES};
use life_pixel_format::{DecodeError, Payload};
use support::{README_EXAMPLE, example_with, example_with_u16, offset};

fn refusal(payload: &[u8]) -> Option<DecodeError> {
    Payload::parse(payload).err()
}

fn assert_truncations_refused(section: core::ops::Range<usize>) {
    for len in section {
        assert_eq!(
            refusal(&README_EXAMPLE[..len]),
            Some(DecodeError::Malformed),
            "{len} bytes"
        );
    }
}

#[test]
fn a_truncated_palette_is_refused() {
    assert_truncations_refused(offset::PALETTE);
}

#[test]
fn an_empty_palette_is_refused() {
    let payload = example_with_u16(offset::PALETTE_COUNT, 0);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_palette_of_more_than_256_entries_is_refused() {
    let mut payload = example_with_u16(offset::PALETTE_COUNT, 257);
    let entries = vec![0; 255 * 4];
    let after_palette = offset::PALETTE.end;
    payload.splice(after_palette..after_palette, entries);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn an_opaque_entry_0_is_refused() {
    let payload = example_with(offset::PALETTE_ENTRY_0, &[0, 0, 0, 1]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_truncated_title_is_refused() {
    assert_truncations_refused(offset::TITLE_SECTION);
}

#[test]
fn a_title_beyond_its_bound_is_refused() {
    let payload = example_with_u16(offset::TITLE_LEN, MAX_TITLE_BYTES + 1);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn invalid_utf8_in_the_title_is_refused() {
    let payload = example_with(offset::TITLE, &[0xC3, 0x28]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn truncated_tags_are_refused() {
    assert_truncations_refused(offset::TAGS);
}

#[test]
fn a_tag_count_beyond_its_bound_is_refused() {
    let payload = example_with_u16(offset::TAG_COUNT, MAX_TAGS + 1);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn a_tag_name_beyond_its_bound_is_refused() {
    let payload = example_with(offset::TAG_NAME_LEN, &[MAX_TAG_NAME_BYTES + 1]);
    assert_eq!(refusal(&payload), Some(DecodeError::BeyondBound));
}

#[test]
fn an_empty_tag_name_is_refused() {
    let payload = example_with(offset::TAG_NAME_LEN, &[0]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn invalid_utf8_in_a_tag_name_is_refused() {
    let payload = example_with(offset::TAG_NAME, &[0x69, 0x64, 0xFF, 0x65]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_tag_whose_first_frame_follows_its_last_is_refused() {
    let payload = example_with(offset::TAG_FIRST, &[1, 0, 0, 0]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_tag_whose_last_frame_is_the_frame_count_is_refused() {
    let payload = example_with_u16(offset::TAG_LAST, 2);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}

#[test]
fn a_loop_mode_above_1_is_refused() {
    let payload = example_with(offset::TAG_LOOP_MODE, &[2]);
    assert_eq!(refusal(&payload), Some(DecodeError::Malformed));
}
