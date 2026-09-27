//! The domain limits of `life-pixel-core` fit within the bounds of payload v1, so that every
//! animation the editor accepts exports: checked on the constants, then on animations at the
//! limits, exported and played in wasmi.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::wasm_player::{DONE, WHOLE_ANIMATION, WasmPlayer};
use life_pixel_compiler::export_wasm;
use life_pixel_core::limits::{
    CANVAS_MAX_SIDE, MAX_CEL_PIXELS, MAX_FRAMES, MAX_PALETTE_ENTRIES, MAX_TAGS,
    MIN_FRAME_DURATION_MS, NAME_MAX_CHARS, TAG_NAME_MAX_CHARS,
};
use life_pixel_core::serialize::read_document;
use life_pixel_core::{Animation, render};
use life_pixel_format::bounds;
use serde_json::{Value, json};

/// The most bytes a UTF-8 character takes.
const UTF8_MAX_BYTES: usize = 4;
/// Payload v1's header, then the counts of its palette, title and tags.
const FIXED_BYTES: usize = 16 + 2 + 2 + 2;
/// A palette entry's bytes: red, green, blue, alpha.
const PALETTE_ENTRY_BYTES: usize = 4;
/// A tag's bytes besides its name: first and last frames, loop mode, name length.
const TAG_BYTES: usize = 2 + 2 + 1 + 1;
/// A frame's bytes besides its data: duration, kind, data length.
const FRAME_BYTES: usize = 2 + 1 + 4;
/// The most data bytes of a blank frame: a single RUN with a 5-byte count, and its index.
const BLANK_FRAME_BYTES: usize = 1 + 5 + 1;
/// The most data bytes per pixel of a frame that is not blank: a one-pixel LITERAL.
const PIXEL_BYTES: usize = 2;

#[test]
#[allow(
    clippy::assertions_on_constants,
    reason = "a test that fails names the limit, where a const block would only fail to build"
)]
fn the_domain_limits_fit_within_the_format_bounds() {
    assert!(CANVAS_MAX_SIDE <= bounds::MAX_SIDE);
    assert!(MAX_FRAMES <= usize::from(bounds::MAX_FRAMES));
    assert!(MAX_TAGS <= usize::from(bounds::MAX_TAGS));
    assert!(
        MAX_PALETTE_ENTRIES <= usize::from(u8::MAX) + 1,
        "an index is a byte"
    );
    assert!(NAME_MAX_CHARS * UTF8_MAX_BYTES <= usize::from(bounds::MAX_TITLE_BYTES));
    // A tag name is ASCII: a character is a byte.
    assert!(TAG_NAME_MAX_CHARS <= usize::from(bounds::MAX_TAG_NAME_BYTES));
    assert!(MIN_FRAME_DURATION_MS >= 1);
}

#[test]
fn the_largest_payload_core_allows_fits_within_the_format_bound() {
    // Only visible cels paint a frame, each covering the canvas: the frames that are not blank
    // hold at most `MAX_CEL_PIXELS` pixels together, and a key frame never takes more than
    // `PIXEL_BYTES` per pixel. A delta frame is only kept when it is smaller.
    let palette = MAX_PALETTE_ENTRIES * PALETTE_ENTRY_BYTES;
    let title = NAME_MAX_CHARS * UTF8_MAX_BYTES;
    let tags = MAX_TAGS * (TAG_BYTES + TAG_NAME_MAX_CHARS);
    let frames = MAX_FRAMES * (FRAME_BYTES + BLANK_FRAME_BYTES) + MAX_CEL_PIXELS * PIXEL_BYTES;

    let largest = FIXED_BYTES + palette + title + tags + frames;

    assert!(largest <= usize::try_from(bounds::MAX_PAYLOAD_BYTES).unwrap());
}

fn document(animation: &Value) -> Animation {
    read_document(animation.to_string().as_bytes()).unwrap()
}

/// The most palette entries: transparent, then opaque colours all different.
fn palette() -> Vec<String> {
    let colours =
        (1..MAX_PALETTE_ENTRIES).map(|index| format!("#{index:02x}{:02x}80ff", 255 - index));
    ["#00000000".to_owned()]
        .into_iter()
        .chain(colours)
        .collect()
}

/// 1 × 1, with the most frames, tags and palette entries, the longest tag names, and the longest
/// title in characters of 4 bytes.
fn most_of_everything() -> Animation {
    let frame_ids = 2..2 + u32::try_from(MAX_FRAMES).unwrap();
    let frames = frame_ids
        .clone()
        .map(|id| json!({ "id": id, "durationMs": MIN_FRAME_DURATION_MS }));
    let colours = (1..MAX_PALETTE_ENTRIES).cycle();
    let cels = frame_ids.clone().zip(colours).map(
        |(frame, colour)| json!({ "layer": 1, "frame": frame, "grid": [format!("{colour:02x}")] }),
    );
    let span = MAX_FRAMES / MAX_TAGS;
    let tags = (0..MAX_TAGS).map(|index| {
        let name = format!("t{index:0>width$}", width = TAG_NAME_MAX_CHARS - 1);
        let (first, loop_mode) = (index * span, ["loop", "once"][index % 2]);
        json!({ "name": name, "first": first, "last": first + span - 1, "loop": loop_mode })
    });
    document(&json!({
        "format": "life-pixel/animation", "version": 1,
        "title": "𝄞".repeat(NAME_MAX_CHARS), "width": 1, "height": 1, "palette": palette(),
        "layers": [{ "id": 1, "name": "Layer", "visible": true }],
        "frames": frames.collect::<Vec<_>>(), "cels": cels.collect::<Vec<_>>(),
        "tags": tags.collect::<Vec<_>>(), "nextId": frame_ids.end,
    }))
}

/// The largest canvas, one frame, its pixels cycling through the palette's colours.
fn largest_canvas() -> Animation {
    let side = usize::from(CANVAS_MAX_SIDE);
    let rows: Vec<String> = (0..side)
        .map(|y| {
            (0..side)
                .map(|x| format!("{:02x}", 1 + (x * 7 + y * 3) % 255))
                .collect()
        })
        .collect();
    document(&json!({
        "format": "life-pixel/animation", "version": 1, "title": "Largest",
        "width": side, "height": side, "palette": palette(),
        "layers": [{ "id": 1, "name": "Layer", "visible": true }],
        "frames": [{ "id": 2, "durationMs": 100 }],
        "cels": [{ "layer": 1, "frame": 2, "grid": rows }], "tags": [], "nextId": 3,
    }))
}

#[test]
fn an_animation_with_the_most_frames_tags_and_colours_exports_and_plays() {
    let animation = most_of_everything();
    let mut player = WasmPlayer::load(&export_wasm(&animation).unwrap());

    assert_eq!(player.title(), animation.title().as_str());
    let names = player.tag_names();
    assert_eq!(names.len(), MAX_TAGS);
    assert!(names.iter().all(|name| name.len() == TAG_NAME_MAX_CHARS));
    assert_eq!(player.call("set_tag", (WHOLE_ANIMATION,)), DONE);
    for frame in [0, MAX_FRAMES / 2, MAX_FRAMES - 1] {
        let position = u32::try_from(frame).unwrap();
        assert_eq!(player.call("seek", (position,)), DONE);
        let id = animation.frames()[frame].id();
        assert_eq!(
            player.frame(),
            render::rgba(&animation, id),
            "frame {frame}"
        );
    }
}

#[test]
fn an_animation_on_the_largest_canvas_exports_and_plays() {
    let animation = largest_canvas();
    let mut player = WasmPlayer::load(&export_wasm(&animation).unwrap());

    let side = u32::from(CANVAS_MAX_SIDE);
    assert_eq!(
        (player.call("width", ()), player.call("height", ())),
        (side, side)
    );
    let id = animation.frames()[0].id();
    assert!(player.frame() == render::rgba(&animation, id));
}
