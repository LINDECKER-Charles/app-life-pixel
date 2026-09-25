//! The v1 fixture: `sample.lpix` decodes to `sample.expected.json`. Neither file ever changes;
//! a new format version adds its own fixture beside them.

use life_pixel_format::{
    ABI_VERSION, FORMAT_VERSION, FrameKind, LoopMode, Payload, Rgba, apply_frame,
};
use serde_json::{Value, json};

const SAMPLE: &[u8] = include_bytes!("fixtures/v1/sample.lpix");
const EXPECTED: &str = include_str!("fixtures/v1/sample.expected.json");

fn header(payload: &Payload<'_>) -> Value {
    json!({
        "format_version": FORMAT_VERSION,
        "abi_version": ABI_VERSION,
        "width": payload.width(),
        "height": payload.height(),
        "frame_count": payload.frame_count(),
        "title": payload.title(),
    })
}

fn palette(payload: &Payload<'_>) -> Value {
    let entries = (0..=u8::MAX).map_while(|index| payload.palette_entry(index));
    entries
        .map(|Rgba { r, g, b, a }| json!([r, g, b, a]))
        .collect()
}

fn tags(payload: &Payload<'_>) -> Value {
    let tags = (0..payload.tag_count()).filter_map(|index| payload.tag(index));
    tags.map(|tag| {
        let loop_mode = match tag.loop_mode {
            LoopMode::Loop => "loop",
            LoopMode::Once => "once",
        };
        json!({ "name": tag.name, "first": tag.first, "last": tag.last, "loop_mode": loop_mode })
    })
    .collect()
}

fn frames(payload: &Payload<'_>) -> Value {
    let mut indices = vec![0; usize::from(payload.width()) * usize::from(payload.height())];
    let frames = payload.frames().map(|frame| {
        let kind = match frame.kind {
            FrameKind::Key => "key",
            FrameKind::Delta => "delta",
        };
        let applied = apply_frame(&frame, &mut indices, payload.palette_len());
        json!({
            "duration_ms": frame.duration_ms,
            "kind": kind,
            "indices": applied.map(|()| indices.clone()).ok(),
        })
    });
    frames.collect()
}

#[test]
fn the_sample_has_3_frames_and_2_tags() {
    let payload = Payload::parse(SAMPLE).unwrap();
    assert_eq!(payload.frame_count(), 3);
    assert_eq!(payload.tag_count(), 2);
}

#[test]
fn the_sample_header_and_title_match() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let decoded = header(&Payload::parse(SAMPLE).unwrap());
    for field in [
        "format_version",
        "abi_version",
        "width",
        "height",
        "frame_count",
        "title",
    ] {
        assert_eq!(decoded[field], expected[field], "{field}");
    }
}

#[test]
fn the_sample_palette_matches() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(
        palette(&Payload::parse(SAMPLE).unwrap()),
        expected["palette"]
    );
}

#[test]
fn the_sample_tags_match() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(tags(&Payload::parse(SAMPLE).unwrap()), expected["tags"]);
}

#[test]
fn the_sample_frames_match() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(frames(&Payload::parse(SAMPLE).unwrap()), expected["frames"]);
}
