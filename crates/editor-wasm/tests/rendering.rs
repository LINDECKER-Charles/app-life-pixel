//! A frame as RGBA through `core::render`, and the preview of an operation in progress, which
//! changes nothing.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::{FRAME, LAYER, SIDE, dot, engine, operation, render};
use life_pixel_core::{DEFAULT_PALETTE, FrameId};
use life_pixel_editor_wasm::render::{RenderRequest, RenderedFrame};
use serde_json::json;

/// The bytes of the pixel at (`x`, `y`).
fn pixel(frame: &RenderedFrame, x: usize, y: usize) -> [u8; 4] {
    let start = (y * usize::from(frame.width) + x) * 4;
    frame.pixels[start..start + 4].try_into().unwrap()
}

fn line_from_origin() -> serde_json::Value {
    json!({
        "kind": "line", "layer": LAYER, "frame": FRAME,
        "from": { "x": 0, "y": 0 }, "to": { "x": 3, "y": 0 }, "index": 2,
    })
}

#[test]
fn a_frame_renders_as_rgba_through_the_palette() {
    let mut engine = engine();
    engine.apply(dot(1, 2, 6)).unwrap();

    let frame = engine.render(&render(None)).unwrap();
    assert_eq!(
        (u32::from(frame.width), u32::from(frame.height)),
        (SIDE, SIDE)
    );
    let side = usize::try_from(SIDE).unwrap();
    assert_eq!(frame.pixels.len(), side * side * 4);
    assert_eq!(pixel(&frame, 1, 2), DEFAULT_PALETTE[6].to_bytes());
    assert_eq!(pixel(&frame, 0, 0), [0, 0, 0, 0]);
}

#[test]
fn a_preview_shows_the_operation_without_applying_it() {
    let mut engine = engine();
    engine.apply(dot(0, 3, 1)).unwrap();
    let before = engine.state();
    let document = engine.serialize().unwrap();

    let preview = engine
        .render(&render(Some(operation(line_from_origin()))))
        .unwrap();
    for x in 0..4 {
        assert_eq!(pixel(&preview, x, 0), DEFAULT_PALETTE[2].to_bytes());
    }
    assert_eq!(pixel(&preview, 0, 3), DEFAULT_PALETTE[1].to_bytes());

    assert_eq!(engine.state(), before);
    assert_eq!(engine.serialize().unwrap(), document);
    assert_eq!(
        pixel(&engine.render(&render(None)).unwrap(), 0, 0),
        [0, 0, 0, 0]
    );
}

#[test]
fn a_preview_on_another_frame_or_of_a_structural_operation_leaves_the_frame_as_it_is() {
    let mut engine = engine();
    engine
        .apply(operation(
            json!({ "kind": "addFrame", "position": 1, "durationMs": 100 }),
        ))
        .unwrap();
    let other_frame = engine.state().document.unwrap().frames[1].id;
    let plain = engine.render(&render(None)).unwrap();

    let mut elsewhere = line_from_origin();
    elsewhere["frame"] = json!(other_frame.get());
    assert_eq!(
        engine.render(&render(Some(operation(elsewhere)))).unwrap(),
        plain
    );
    let rename = operation(json!({ "kind": "setTitle", "title": "Renamed" }));
    assert_eq!(engine.render(&render(Some(rename))).unwrap(), plain);
}

#[test]
fn a_missing_frame_or_a_broken_preview_is_refused_with_its_code() {
    let engine = engine();
    let missing = RenderRequest {
        frame: FrameId::new(9_999),
        preview: None,
    };
    assert_eq!(
        engine.render(&missing).unwrap_err().code,
        "edit.frame_not_found"
    );

    let mut wrong_layer = line_from_origin();
    wrong_layer["layer"] = json!(9_999);
    let error = engine
        .render(&render(Some(operation(wrong_layer))))
        .unwrap_err();
    assert_eq!(error.code, "edit.layer_not_found");
}
