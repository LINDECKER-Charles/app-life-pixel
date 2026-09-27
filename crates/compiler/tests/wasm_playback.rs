//! Each fixture's export played in wasmi through ABI v1, as the loader plays it: every frame of the
//! whole animation and of each tag shows the RGBA of `life-pixel-core`'s rendering, and the size,
//! the title, the tag names and their loop modes come back through the ABI.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::wasm_player::{DONE, FRAME_CHANGED, RANGE_ENDED, WHOLE_ANIMATION, WasmPlayer};
use common::{WASM_FIXTURES, animation};
use life_pixel_compiler::export_wasm;
use life_pixel_core::{Animation, LoopMode, render};

/// A range to play: `set_tag`'s index, its first and last frames, and how it ends.
struct Range {
    index: u32,
    first: u32,
    last: u32,
    is_looping: bool,
}

/// The whole animation, which loops, then each tag.
fn ranges(animation: &Animation) -> Vec<Range> {
    let last = u32::try_from(animation.frames().len()).unwrap() - 1;
    let whole = Range {
        index: WHOLE_ANIMATION,
        first: 0,
        last,
        is_looping: true,
    };
    let tags = (0..).zip(animation.tags()).map(|(index, tag)| Range {
        index,
        first: tag.first().into(),
        last: tag.last().into(),
        is_looping: tag.loop_mode() == LoopMode::Loop,
    });
    [whole].into_iter().chain(tags).collect()
}

fn loaded(animation: &Animation) -> WasmPlayer {
    WasmPlayer::load(&export_wasm(animation).unwrap())
}

/// Checks that `frame` is shown, with the RGBA `life-pixel-core` renders for it.
fn expect_frame(player: &mut WasmPlayer, animation: &Animation, frame: u32) {
    assert_eq!(player.call("frame_index", ()), frame, "the frame shown");
    let id = animation.frames()[usize::try_from(frame).unwrap()].id();
    assert!(
        player.frame() == render::rgba(animation, id),
        "frame {frame} of {:?} differs from core's rendering",
        animation.title().as_str()
    );
}

fn duration_ms(animation: &Animation, frame: u32) -> u32 {
    animation.frames()[usize::try_from(frame).unwrap()]
        .duration_ms()
        .into()
}

/// Plays `range` once through, each tick exactly a frame's duration, checking every frame.
fn play(player: &mut WasmPlayer, animation: &Animation, range: &Range) {
    assert_eq!(player.call("set_tag", (range.index,)), DONE);
    for frame in range.first..range.last {
        expect_frame(player, animation, frame);
        let flags = player.call("tick", (duration_ms(animation, frame),));
        assert_eq!(flags, FRAME_CHANGED, "a tick over frame {frame}");
    }
    expect_frame(player, animation, range.last);
}

/// Ticks over the last frame of the current range, then checks where the range's mode leaves it.
fn end_range(player: &mut WasmPlayer, animation: &Animation, range: &Range) {
    let flags = player.call("tick", (duration_ms(animation, range.last),));
    assert!(
        flags & RANGE_ENDED != 0,
        "the range ends after its last frame"
    );
    if range.is_looping {
        expect_frame(player, animation, range.first);
    } else {
        expect_frame(player, animation, range.last);
        assert_eq!(
            player.call("tick", (u32::MAX,)),
            0,
            "a range played once stops"
        );
    }
}

#[test]
fn every_frame_of_every_range_shows_what_core_renders() {
    for document in WASM_FIXTURES {
        let animation = animation(document);
        let mut player = loaded(&animation);
        let initial_frame = animation.tags().first().map_or(0, |tag| tag.first().into());
        expect_frame(&mut player, &animation, initial_frame);
        for range in ranges(&animation) {
            play(&mut player, &animation, &range);
        }
    }
}

#[test]
fn the_size_title_and_tag_names_come_back_through_the_abi() {
    for document in WASM_FIXTURES {
        let animation = animation(document);
        let mut player = loaded(&animation);

        let size = (player.call("width", ()), player.call("height", ()));
        let expected = (animation.width().into(), animation.height().into());
        assert_eq!(size, expected);
        assert_eq!(player.title(), animation.title().as_str());
        let names: Vec<&str> = animation
            .tags()
            .iter()
            .map(|tag| tag.name().as_str())
            .collect();
        assert_eq!(player.tag_names(), names);
    }
}

#[test]
fn a_looping_range_starts_again_and_a_range_played_once_stops_on_its_last_frame() {
    let mut modes = Vec::new();
    for document in WASM_FIXTURES {
        let animation = animation(document);
        let mut player = loaded(&animation);
        for range in ranges(&animation) {
            assert_eq!(player.call("set_tag", (range.index,)), DONE);
            assert_eq!(player.call("seek", (range.last - range.first,)), DONE);
            end_range(&mut player, &animation, &range);
            modes.push(range.is_looping);
        }
    }
    assert!(
        modes.contains(&true) && modes.contains(&false),
        "both modes are played"
    );
}
