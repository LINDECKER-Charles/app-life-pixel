use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use life_pixel_format::{AnimationData, DecodeError, FrameData, Rgba, encode};

use super::*;
use crate::{FRAME_CHANGED, RANGE_ENDED, RANGE_STOPPED};

/// The v1 fixture: 12 × 8, 3 frames of 120, 120 and 240 ms, titled `Sample`, with the tags
/// `idle` (frames 0 to 1, looping) and `flash` (frame 2, once).
const SAMPLE: &[u8] = include_bytes!("../../../format/tests/fixtures/v1/sample.lpix");

fn len(payload: &[u8]) -> u32 {
    u32::try_from(payload.len()).unwrap()
}

/// Reserves a block for `payload` and writes it there, as the loader does; returns its address.
fn write(player: &mut Player, payload: &[u8]) -> usize {
    let block = player.alloc(len(payload)).unwrap();
    block.copy_from_slice(payload);
    block.as_ptr().addr()
}

fn loaded(payload: &[u8]) -> Player {
    let mut player = Player::new();
    let address = write(&mut player, payload);
    player.load(address, len(payload)).unwrap();
    player
}

/// Three frames of 1 × 1 pixel, 10 ms each, without tags.
fn untagged() -> Vec<u8> {
    let transparent = Rgba::default();
    let frame = |index| FrameData {
        duration_ms: 10,
        indices: vec![index],
    };
    encode(&AnimationData {
        width: 1,
        height: 1,
        palette: vec![
            transparent,
            Rgba {
                a: 255,
                ..transparent
            },
        ],
        title: String::new(),
        tags: Vec::new(),
        frames: vec![frame(0), frame(1), frame(0)],
    })
    .unwrap()
}

fn offset_of(needle: &[u8]) -> usize {
    SAMPLE
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap()
}

#[test]
fn every_export_waits_for_a_successful_load() {
    let mut player = Player::new();

    assert_eq!(player.load(0, 0), Err(CallError::WrongCallOrder));
    assert_eq!(player.set_tag(0), Err(CallError::WrongCallOrder));
    assert_eq!(player.set_loop(0), Err(CallError::WrongCallOrder));
    assert_eq!(player.seek(0), Err(CallError::WrongCallOrder));
    assert_eq!(player.tick(1_000), 0);
    let queries = [
        player.width(),
        player.height(),
        player.tag_count(),
        player.frame_index(),
    ];
    assert_eq!(queries, [0; 4]);
    let addresses = [
        player.frame_ptr(),
        player.tag_name_ptr(0),
        player.title_ptr(),
    ];
    assert_eq!(addresses, [0; 3]);
    assert_eq!([player.tag_name_len(0), player.title_len()], [0; 2]);
}

#[test]
fn abi_version_is_1() {
    assert_eq!(Player::new().abi_version(), 1);
}

#[test]
fn alloc_reserves_once_per_instance() {
    let mut player = Player::new();

    assert_eq!(player.alloc(16).map(|block| block.len()), Some(16));
    assert!(player.alloc(16).is_none());
}

#[test]
fn alloc_refuses_more_than_the_largest_payload_and_spends_the_instance() {
    let mut player = Player::new();

    assert!(player.alloc(MAX_PAYLOAD_BYTES + 1).is_none());
    assert!(player.alloc(16).is_none());
    assert_eq!(player.load(0, 0), Err(CallError::WrongCallOrder));
}

#[test]
fn load_takes_the_reserved_block_only() {
    let mut player = Player::new();
    let address = write(&mut player, SAMPLE);

    assert_eq!(
        player.load(address + 1, len(SAMPLE)),
        Err(CallError::ArgumentOutOfRange)
    );
    assert_eq!(
        player.load(address, len(SAMPLE) + 1),
        Err(CallError::ArgumentOutOfRange)
    );
    assert_eq!(player.load(address, len(SAMPLE)), Ok(()));
}

#[test]
fn load_shows_the_first_frame_of_tag_0() {
    let player = loaded(SAMPLE);

    assert_eq!([player.width(), player.height()], [12, 8]);
    assert_eq!(player.tag_count(), 2);
    assert_eq!(player.frame_index(), 0);
    assert_ne!(player.frame_ptr(), 0);
}

#[test]
fn load_twice_is_refused_and_playback_goes_on() {
    let mut player = Player::new();
    let address = write(&mut player, SAMPLE);
    player.load(address, len(SAMPLE)).unwrap();

    assert_eq!(
        player.load(address, len(SAMPLE)),
        Err(CallError::WrongCallOrder)
    );
    assert_eq!(player.tick(120), FRAME_CHANGED);
}

#[test]
fn a_refused_payload_returns_the_decoder_status_and_spends_the_instance() {
    let mut player = Player::new();
    let mut payload = SAMPLE.to_vec();
    payload[4] = 2;
    let address = write(&mut player, &payload);

    let refused = player.load(address, len(&payload));

    assert_eq!(
        refused,
        Err(CallError::Payload(DecodeError::UnknownFormatVersion))
    );
    assert_eq!(refused.unwrap_err().status(), 2);
    assert_eq!(
        player.load(address, len(&payload)),
        Err(CallError::WrongCallOrder)
    );
    assert_eq!(player.width(), 0);
}

#[test]
fn a_prefix_of_the_block_is_a_truncated_payload() {
    let mut player = Player::new();
    let address = write(&mut player, SAMPLE);

    let refused = player.load(address, len(SAMPLE) - 1);

    assert_eq!(refused, Err(CallError::Payload(DecodeError::Malformed)));
}

#[test]
fn the_title_and_tag_names_point_into_the_payload() {
    let mut player = Player::new();
    let address = write(&mut player, SAMPLE);
    player.load(address, len(SAMPLE)).unwrap();

    assert_eq!(player.title_ptr(), address + offset_of(b"Sample"));
    assert_eq!(player.title_len(), 6);
    assert_eq!(player.tag_name_ptr(1), address + offset_of(b"flash"));
    assert_eq!([player.tag_name_len(0), player.tag_name_len(1)], [4, 5]);
    assert_eq!(
        [player.tag_name_ptr(2), player.tag_name_ptr(u32::MAX)],
        [0, 0]
    );
    assert_eq!(player.tag_name_len(2), 0);
}

#[test]
fn the_initial_tag_loops_over_its_frames() {
    let mut player = loaded(SAMPLE);

    assert_eq!(player.tick(120), FRAME_CHANGED);
    assert_eq!(player.frame_index(), 1);
    assert_eq!(player.tick(120), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(player.frame_index(), 0);
}

#[test]
fn set_tag_plays_a_tag_from_its_first_frame_in_its_own_mode() {
    let mut player = loaded(SAMPLE);

    assert_eq!(player.set_tag(1), Ok(()));
    assert_eq!(player.frame_index(), 2);
    assert_eq!(player.tick(240), RANGE_ENDED | RANGE_STOPPED);
    assert_eq!(player.tick(240), 0);
    assert_eq!(player.set_tag(2), Err(CallError::ArgumentOutOfRange));
    assert_eq!(player.frame_index(), 2);
}

#[test]
fn set_tag_0xffffffff_plays_the_whole_animation() {
    let mut player = loaded(SAMPLE);
    player.seek(1).unwrap();

    assert_eq!(player.set_tag(u32::MAX), Ok(()));
    assert_eq!(player.frame_index(), 0);
    assert_eq!(player.tick(120 + 120), FRAME_CHANGED);
    assert_eq!(player.frame_index(), 2);
    assert_eq!(player.tick(240), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(player.frame_index(), 0);
}

#[test]
fn set_loop_checks_its_mode_and_applies_at_once() {
    let mut player = loaded(SAMPLE);
    player.set_tag(1).unwrap();

    assert_eq!(player.set_loop(3), Err(CallError::ArgumentOutOfRange));
    assert_eq!(player.set_loop(1), Ok(()));
    assert_eq!(player.tick(240), RANGE_ENDED);
    assert_eq!(player.tick(240), RANGE_ENDED);
}

#[test]
fn seek_counts_from_the_range_first_frame_and_restarts_it() {
    let mut player = loaded(SAMPLE);
    player.set_tag(1).unwrap();
    player.tick(240);

    assert_eq!(player.seek(1), Err(CallError::ArgumentOutOfRange));
    assert_eq!(player.seek(0), Ok(()));
    assert_eq!(player.frame_index(), 2);
    assert_eq!(player.tick(240), RANGE_ENDED | RANGE_STOPPED);
}

#[test]
fn an_animation_without_tags_plays_whole_and_loops() {
    let mut player = loaded(&untagged());

    assert_eq!(player.tag_count(), 0);
    assert_eq!(player.frame_index(), 0);
    assert_eq!(player.tick(10), FRAME_CHANGED);
    assert_eq!(player.tick(10), FRAME_CHANGED);
    assert_eq!(player.tick(10), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(player.frame_index(), 0);
    assert_eq!(player.set_tag(0), Err(CallError::ArgumentOutOfRange));
}
