//! Zips of PNG frames, read back: names, order, storage, date, and each frame's pixels.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::io::{Cursor, Read};

use common::{BLINK, BLINK_FRAMES, MASCOT, MASCOT_FRAMES, animation, options};
use life_pixel_compiler::{ClassicOptions, export_png_frames};
use png::{ColorType, Decoder, Transformations};
use zip::{CompressionMethod, DateTime, ZipArchive};

struct Entry {
    name: String,
    compression: CompressionMethod,
    modified: Option<DateTime>,
    bytes: Vec<u8>,
}

fn entries(zip: &[u8]) -> Vec<Entry> {
    let mut archive = ZipArchive::new(Cursor::new(zip)).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut file = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            Entry {
                name: file.name().to_owned(),
                compression: file.compression(),
                modified: file.last_modified(),
                bytes,
            }
        })
        .collect()
}

/// The size and the palette indices of an indexed PNG.
fn pixels(png: &[u8]) -> ((u32, u32), Vec<u8>) {
    let mut decoder = Decoder::new(Cursor::new(png));
    decoder.set_transformations(Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    assert_eq!(reader.info().color_type, ColorType::Indexed);
    assert!(reader.info().trns.is_some());
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buffer).unwrap();
    buffer.truncate(frame.buffer_size());
    ((frame.width, frame.height), buffer)
}

#[test]
fn every_frame_is_a_stored_png_dated_1980_in_play_order() {
    let file = export_png_frames(&animation(MASCOT), &ClassicOptions::default()).unwrap();
    assert_eq!(file.name, "mascot-frames.zip");
    let entries = entries(&file.bytes);
    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, ["mascot-0.png", "mascot-1.png", "mascot-2.png"]);
    for (entry, expected) in entries.iter().zip(MASCOT_FRAMES) {
        assert_eq!(entry.compression, CompressionMethod::Stored);
        let modified = entry.modified.unwrap();
        let date = (modified.year(), modified.month(), modified.day());
        assert_eq!(date, (1980, 1, 1));
        assert_eq!(pixels(&entry.bytes), ((4, 3), expected.to_vec()));
    }
}

#[test]
fn a_tag_and_a_scale_export_its_frames_enlarged() {
    let file = export_png_frames(&animation(BLINK), &options(Some("blink"), 2)).unwrap();
    let entries = entries(&file.bytes);
    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, ["blink-2-0.png", "blink-2-1.png", "blink-2-2.png"]);
    let (size, indices) = pixels(&entries[2].bytes);
    assert_eq!(size, (4, 4));
    assert_eq!(indices, [BLINK_FRAMES[3][0]; 16]);
}
