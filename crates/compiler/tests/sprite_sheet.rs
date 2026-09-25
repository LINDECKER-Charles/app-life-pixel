//! Sprite sheets, read back: the grid of frames in the PNG, and the Aseprite "array" JSON.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::io::Cursor;

use common::{BLINK, MASCOT, MASCOT_FRAMES, animation, options};
use life_pixel_compiler::{ClassicOptions, ExportFile, export_sprite_sheet};
use png::{ColorType, Decoder, Transformations};
use serde_json::{Value, json};

fn export(document: &str, options: &ClassicOptions) -> (ExportFile, Value) {
    let mut files = export_sprite_sheet(&animation(document), options).unwrap();
    let json = files.pop().unwrap();
    assert_eq!(json.media_type, "application/json");
    (
        files.pop().unwrap(),
        serde_json::from_slice(&json.bytes).unwrap(),
    )
}

/// The size and the palette indices of an indexed PNG.
fn pixels(png: &[u8]) -> ((u32, u32), Vec<u8>) {
    let mut decoder = Decoder::new(Cursor::new(png));
    decoder.set_transformations(Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    assert_eq!(reader.info().color_type, ColorType::Indexed);
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buffer).unwrap();
    buffer.truncate(frame.buffer_size());
    ((frame.width, frame.height), buffer)
}

/// The pixels of `[left, top, width, height]` on a sheet `sheet_width` pixels wide.
fn cell(sheet: &[u8], sheet_width: usize, rectangle: [usize; 4]) -> Vec<u8> {
    let [left, top, width, height] = rectangle;
    let rows = (top..top + height).map(|row| row * sheet_width + left);
    rows.flat_map(|start| &sheet[start..start + width])
        .copied()
        .collect()
}

#[test]
fn frames_fill_a_grid_of_ceil_sqrt_n_columns_row_by_row() {
    let (image, _) = export(MASCOT, &ClassicOptions::default());
    assert_eq!(image.name, "mascot.png");
    let ((width, height), sheet) = pixels(&image.bytes);
    assert_eq!((width, height), (8, 6));
    let origins = [(0, 0), (4, 0), (0, 3)];
    for ((left, top), expected) in origins.into_iter().zip(MASCOT_FRAMES) {
        assert_eq!(cell(&sheet, 8, [left, top, 4, 3]), expected);
    }
    assert_eq!(
        cell(&sheet, 8, [4, 3, 4, 3]),
        [0; 12],
        "the unused cell is transparent"
    );
}

#[test]
fn the_json_follows_the_aseprite_array_layout() {
    let (_, description) = export(MASCOT, &ClassicOptions::default());
    let first = json!({
        "filename": "mascot 0", "frame": { "x": 0, "y": 0, "w": 4, "h": 3 }, "rotated": false,
        "trimmed": false, "spriteSourceSize": { "x": 0, "y": 0, "w": 4, "h": 3 },
        "sourceSize": { "w": 4, "h": 3 }, "duration": 100
    });
    assert_eq!(description["frames"][0], first);
    let rectangles: Vec<&Value> = description["frames"].as_array().unwrap().iter().collect();
    let names: Vec<&Value> = rectangles.iter().map(|frame| &frame["filename"]).collect();
    assert_eq!(names, ["mascot 0", "mascot 1", "mascot 2"]);
    assert_eq!(
        rectangles[2]["frame"],
        json!({ "x": 0, "y": 3, "w": 4, "h": 3 })
    );
    assert_eq!(rectangles[2]["duration"], 25);
    let meta = json!({
        "app": "Life Pixel", "version": "1", "image": "mascot.png", "format": "RGBA8888",
        "size": { "w": 8, "h": 6 }, "scale": "1",
        "frameTags": [
            { "name": "idle", "from": 0, "to": 1, "direction": "forward" },
            { "name": "jump", "from": 2, "to": 2, "direction": "forward", "repeat": "1" }
        ]
    });
    assert_eq!(description["meta"], meta);
}

#[test]
fn a_tag_and_a_scale_export_its_frames_enlarged_with_the_tags_it_holds() {
    let (image, description) = export(BLINK, &options(Some("blink"), 2));
    let ((width, height), sheet) = pixels(&image.bytes);
    assert_eq!((width, height), (8, 8));
    assert_eq!(cell(&sheet, 8, [0, 4, 4, 4]), [2; 16]);
    assert_eq!(description["frames"].as_array().unwrap().len(), 3);
    assert_eq!(description["frames"][1]["filename"], "blink-2 1");
    assert_eq!(
        description["frames"][1]["frame"],
        json!({ "x": 4, "y": 0, "w": 4, "h": 4 })
    );
    assert_eq!(description["meta"]["scale"], "2");
    let tag = json!({ "name": "blink", "from": 0, "to": 2, "direction": "forward", "repeat": "1" });
    assert_eq!(description["meta"]["frameTags"], json!([tag]));
    let (_, idle) = export(MASCOT, &options(Some("idle"), 1));
    let names: Vec<&Value> = idle["meta"]["frameTags"]
        .as_array()
        .unwrap()
        .iter()
        .collect();
    assert_eq!(
        names,
        [&json!({ "name": "idle", "from": 0, "to": 1, "direction": "forward" })]
    );
}
