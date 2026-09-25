//! The v1 fixture played in the built player, as the loader plays it: the header through the
//! ABI, then the whole animation and each tag, frame by frame, each framebuffer compared with the
//! expected indices through the palette.

use std::path::Path;

use anyhow::{Context, ensure};

use super::fixture::Fixture;
use super::wasm_player::{DONE, WasmPlayer};

/// The ABI the player must implement.
const ABI_VERSION: u32 = 1;

/// `set_tag`'s index for the whole animation.
const WHOLE_ANIMATION: u32 = u32::MAX;

/// `tick`'s bit 0: the framebuffer changed.
const FRAME_CHANGED: u32 = 1;

/// `tick`'s bit 1: the range reached its end.
const RANGE_ENDED: u32 = 2;

/// A range to play: `set_tag`'s index, its first and last frames, and how it ends.
struct Range {
    index: u32,
    first: u32,
    last: u32,
    is_looping: bool,
}

/// Plays the fixture of the workspace at `root` in `module`; returns what was played.
pub fn check(root: &Path, module: &[u8]) -> anyhow::Result<String> {
    let fixture = Fixture::read(root)?;
    let mut player = WasmPlayer::instantiate(module)?;
    let version = player.call("abi_version", ())?;
    ensure!(version == ABI_VERSION, "abi_version returned {version}");
    player.load(&fixture.payload)?;
    check_header(&mut player, &fixture)?;
    let initial_frame = fixture.tags.first().map_or(0, |tag| tag.first);
    expect_frame(&mut player, &fixture, initial_frame).context("after load")?;
    let ranges = ranges(&fixture)?;
    let mut frame_count = 0;
    for range in &ranges {
        let context = || format!("playing set_tag({:#x})", range.index);
        frame_count += play(&mut player, &fixture, range).with_context(context)?;
    }
    Ok(format!(
        "{frame_count} frames played over {} ranges, no import",
        ranges.len()
    ))
}

/// The size, the title and the tag names, read through the ABI.
fn check_header(player: &mut WasmPlayer, fixture: &Fixture) -> anyhow::Result<()> {
    let size = (player.call("width", ())?, player.call("height", ())?);
    ensure!(
        size == (fixture.width, fixture.height),
        "the canvas is {} × {}",
        size.0,
        size.1
    );
    let (pointer, len) = (player.call("title_ptr", ())?, player.call("title_len", ())?);
    let title = text(player, pointer, len)?;
    ensure!(title == fixture.title, "the title is {title:?}");
    let tag_count = player.call("tag_count", ())?;
    ensure!(usize::try_from(tag_count)? == fixture.tags.len(), "{tag_count} tags");
    for (index, tag) in (0..).zip(&fixture.tags) {
        let pointer = player.call("tag_name_ptr", (index,))?;
        let len = player.call("tag_name_len", (index,))?;
        let name = text(player, pointer, len)?;
        ensure!(name == tag.name, "tag {index} is named {name:?}");
    }
    Ok(())
}

/// The whole animation, which loops, then each tag.
fn ranges(fixture: &Fixture) -> anyhow::Result<Vec<Range>> {
    let last = u32::try_from(fixture.frames.len())? - 1;
    let whole = Range {
        index: WHOLE_ANIMATION,
        first: 0,
        last,
        is_looping: true,
    };
    let tags = (0..).zip(&fixture.tags).map(|(index, tag)| Range {
        index,
        first: tag.first,
        last: tag.last,
        is_looping: tag.is_looping,
    });
    Ok([whole].into_iter().chain(tags).collect())
}

/// Plays `range` once through, each tick exactly a frame's duration; returns its frame count.
fn play(player: &mut WasmPlayer, fixture: &Fixture, range: &Range) -> anyhow::Result<u32> {
    let status = player.call("set_tag", (range.index,))?;
    ensure!(status == DONE, "set_tag returned status {status}");
    for frame in range.first..=range.last {
        expect_frame(player, fixture, frame)?;
        let flags = player.call("tick", (duration_ms(fixture, frame)?,))?;
        let expected = expected_flags(range, frame);
        ensure!(flags == expected, "tick over frame {frame} returned {flags}, not {expected}");
    }
    let end_frame = if range.is_looping { range.first } else { range.last };
    expect_frame(player, fixture, end_frame).context("after the range's end")?;
    if !range.is_looping {
        let flags = player.call("tick", (u32::MAX,))?;
        ensure!(flags == 0, "a range played once still ticks: {flags}");
    }
    Ok(range.last - range.first + 1)
}

/// What a tick over the whole of `frame` returns: the next frame, or the range's end — which
/// changes the frame when it loops back to another one.
fn expected_flags(range: &Range, frame: u32) -> u32 {
    let is_last = frame == range.last;
    let changes_frame = !is_last || (range.is_looping && range.first != range.last);
    (u32::from(changes_frame) * FRAME_CHANGED) | (u32::from(is_last) * RANGE_ENDED)
}

/// Checks that `frame` is shown, and that the framebuffer holds its expected pixels.
fn expect_frame(player: &mut WasmPlayer, fixture: &Fixture, frame: u32) -> anyhow::Result<()> {
    let shown = player.call("frame_index", ())?;
    ensure!(shown == frame, "frame {shown} is shown, not frame {frame}");
    let expected = &expected(fixture, frame)?.rgba;
    let pointer = player.call("frame_ptr", ())?;
    let rgba = player.read(pointer, u32::try_from(expected.len())?)?;
    ensure!(
        &rgba == expected,
        "the framebuffer of frame {frame} differs from its expected indices through the palette"
    );
    Ok(())
}

fn duration_ms(fixture: &Fixture, frame: u32) -> anyhow::Result<u32> {
    Ok(expected(fixture, frame)?.duration_ms)
}

fn expected(fixture: &Fixture, frame: u32) -> anyhow::Result<&super::fixture::Frame> {
    let index = usize::try_from(frame)?;
    fixture
        .frames
        .get(index)
        .with_context(|| format!("the fixture has no frame {frame}"))
}

fn text(player: &WasmPlayer, pointer: u32, len: u32) -> anyhow::Result<String> {
    Ok(String::from_utf8(player.read(pointer, len)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(first: u32, last: u32, is_looping: bool) -> Range {
        Range {
            index: 0,
            first,
            last,
            is_looping,
        }
    }

    #[test]
    fn a_tick_over_a_frame_changes_it_until_the_range_ends() {
        let looping = range(0, 2, true);

        assert_eq!(expected_flags(&looping, 1), FRAME_CHANGED);
        assert_eq!(expected_flags(&looping, 2), FRAME_CHANGED | RANGE_ENDED);
        assert_eq!(expected_flags(&range(0, 2, false), 2), RANGE_ENDED);
        assert_eq!(expected_flags(&range(2, 2, true), 2), RANGE_ENDED);
    }
}
