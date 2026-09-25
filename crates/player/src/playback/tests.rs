use core::cell::Cell;

use super::*;

/// Four frames of 100, 50, 200 and 80 ms: 430 ms in all.
const DURATIONS: [u16; 4] = [100, 50, 200, 80];
const WHOLE_MS: u32 = 430;

fn duration_ms(frame: u16) -> u16 {
    DURATIONS[usize::from(frame)]
}

fn range(first: u16, last: u16, loop_mode: LoopMode) -> Range {
    Range {
        first,
        last,
        loop_mode,
    }
}

fn playing(range: Range) -> Playback {
    Playback::new(range, duration_ms)
}

fn whole() -> Playback {
    playing(range(0, 3, LoopMode::Loop))
}

#[test]
fn a_tick_shorter_than_the_frame_changes_nothing() {
    let mut playback = whole();

    assert_eq!(playback.tick(99, duration_ms), 0);
    assert_eq!(playback.frame(), 0);
}

#[test]
fn time_adds_up_until_it_covers_the_frame() {
    let mut playback = whole();
    playback.tick(99, duration_ms);

    assert_eq!(playback.tick(1, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.frame(), 1);
}

#[test]
fn one_tick_can_cover_several_frames() {
    let mut playback = whole();

    assert_eq!(playback.tick(100 + 50 + 10, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.frame(), 2);
    assert_eq!(playback.tick(189, duration_ms), 0);
    assert_eq!(playback.tick(1, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.frame(), 3);
}

#[test]
fn a_looping_range_goes_back_to_its_first_frame() {
    let mut playback = playing(range(1, 2, LoopMode::Loop));
    playback.tick(50, duration_ms);

    assert_eq!(playback.tick(200, duration_ms), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(playback.frame(), 1);
}

#[test]
fn bit_1_is_set_on_each_cycle_and_only_then() {
    let mut playback = whole();

    for _ in 0..3 {
        for elapsed_ms in [100, 50, 200] {
            assert_eq!(playback.tick(elapsed_ms, duration_ms), FRAME_CHANGED);
        }
        assert_eq!(playback.tick(80, duration_ms), FRAME_CHANGED | RANGE_ENDED);
        assert_eq!(playback.frame(), 0);
    }
}

#[test]
fn a_range_played_once_stops_on_its_last_frame() {
    let mut playback = playing(range(0, 1, LoopMode::Once));

    assert_eq!(playback.tick(100, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.tick(50, duration_ms), RANGE_ENDED);
    assert_eq!(playback.frame(), 1);
    assert_eq!(playback.tick(10_000, duration_ms), 0);
    assert_eq!(playback.frame(), 1);
}

#[test]
fn a_range_played_once_stops_within_a_single_long_tick() {
    let mut playback = playing(range(0, 3, LoopMode::Once));

    assert_eq!(
        playback.tick(u32::MAX, duration_ms),
        FRAME_CHANGED | RANGE_ENDED
    );
    assert_eq!(playback.frame(), 3);
}

#[test]
fn a_single_frame_range_ends_without_changing_the_frame() {
    let mut playback = playing(range(2, 2, LoopMode::Loop));

    assert_eq!(playback.tick(200, duration_ms), RANGE_ENDED);
    assert_eq!(playback.frame(), 2);
}

#[test]
fn a_long_pause_lands_where_its_remainder_does() {
    let mut playback = whole();

    let flags = playback.tick(WHOLE_MS * 9_000_000 + 160, duration_ms);

    assert_eq!(flags, FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(playback.frame(), 2);
    assert_eq!(playback.tick(189, duration_ms), 0);
}

#[test]
fn a_long_pause_costs_at_most_one_cycle() {
    let mut playback = whole();
    let lookups = Cell::new(0);
    let counted = |frame| {
        lookups.set(lookups.get() + 1);
        duration_ms(frame)
    };

    playback.tick(u32::MAX, counted);

    assert!(
        lookups.get() <= 2 * DURATIONS.len() + 1,
        "{}",
        lookups.get()
    );
}

#[test]
fn set_range_plays_the_new_range_from_its_first_frame() {
    let mut playback = playing(range(0, 0, LoopMode::Once));
    playback.tick(100, duration_ms);

    playback.set_range(range(2, 3, LoopMode::Loop), duration_ms);

    assert_eq!(playback.frame(), 2);
    assert_eq!(playback.tick(200, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.tick(80, duration_ms), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(playback.frame(), 2);
}

#[test]
fn seek_shows_a_frame_counted_from_the_range_start() {
    let mut playback = playing(range(1, 3, LoopMode::Loop));

    assert_eq!(playback.seek(2), Ok(()));
    assert_eq!(playback.frame(), 3);
    assert_eq!(playback.seek(3), Err(CallError::ArgumentOutOfRange));
    assert_eq!(playback.seek(u32::MAX), Err(CallError::ArgumentOutOfRange));
    assert_eq!(playback.frame(), 3);
}

#[test]
fn seek_restarts_a_stopped_range_and_its_time() {
    let mut playback = playing(range(0, 1, LoopMode::Once));
    playback.tick(1_000, duration_ms);
    assert_eq!(playback.tick(1_000, duration_ms), 0);

    assert_eq!(playback.seek(0), Ok(()));

    assert_eq!(playback.tick(99, duration_ms), 0);
    assert_eq!(playback.tick(1, duration_ms), FRAME_CHANGED);
}

#[test]
fn set_loop_overrides_the_range_mode_until_set_back() {
    let mut playback = playing(range(0, 1, LoopMode::Once));

    playback.set_loop(LoopSetting::Loop);
    assert_eq!(playback.tick(100, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.tick(50, duration_ms), FRAME_CHANGED | RANGE_ENDED);
    assert_eq!(playback.frame(), 0);

    playback.set_loop(LoopSetting::Own);
    assert_eq!(playback.tick(100, duration_ms), FRAME_CHANGED);
    assert_eq!(playback.tick(50, duration_ms), RANGE_ENDED);
    assert_eq!(playback.frame(), 1);
    assert_eq!(playback.tick(150, duration_ms), 0);
}

#[test]
fn set_loop_once_stops_a_looping_range_at_its_next_end() {
    let mut playback = whole();
    playback.tick(100, duration_ms);

    playback.set_loop(LoopSetting::Once);

    assert_eq!(
        playback.tick(u32::MAX, duration_ms),
        FRAME_CHANGED | RANGE_ENDED
    );
    assert_eq!(playback.frame(), 3);
}

#[test]
fn set_loop_leaves_a_stopped_range_stopped() {
    let mut playback = playing(range(0, 1, LoopMode::Once));
    playback.tick(150, duration_ms);

    playback.set_loop(LoopSetting::Loop);

    assert_eq!(playback.tick(1_000, duration_ms), 0);
    assert_eq!(playback.frame(), 1);
}

#[test]
fn loop_modes_are_0_own_1_loop_2_once() {
    let settings = [0, 1, 2, 3].map(LoopSetting::from_mode);

    assert_eq!(
        settings,
        [
            Ok(LoopSetting::Own),
            Ok(LoopSetting::Loop),
            Ok(LoopSetting::Once),
            Err(CallError::ArgumentOutOfRange),
        ]
    );
}
