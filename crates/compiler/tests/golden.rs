//! Every classic export of the two fixture animations, byte for byte against `tests/golden/`.
//! After a deliberate change of output, rewrite them with
//! `LIFE_PIXEL_UPDATE_GOLDEN=1 cargo test -p life-pixel-compiler --test golden`, then review the
//! decoding tests, which say what the bytes must show.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::path::PathBuf;

use common::{BLINK, MASCOT, animation, options};
use life_pixel_compiler::{
    ExportError, ExportFile, export_apng, export_gif, export_png_frames, export_sprite_sheet,
};
use life_pixel_core::Animation;

const UPDATE_VARIABLE: &str = "LIFE_PIXEL_UPDATE_GOLDEN";

fn every_export(animation: &Animation) -> Result<Vec<ExportFile>, ExportError> {
    let options = options(None, 1);
    let mut files = vec![
        export_gif(animation, &options)?,
        export_apng(animation, &options)?,
    ];
    files.extend(export_sprite_sheet(animation, &options)?);
    files.push(export_png_frames(animation, &options)?);
    Ok(files)
}

fn golden_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "golden", name]
        .iter()
        .collect()
}

fn check_against_golden(file: &ExportFile) {
    let path = golden_path(&file.name);
    if std::env::var_os(UPDATE_VARIABLE).is_some() {
        std::fs::write(&path, &file.bytes).unwrap();
    }
    let golden = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    assert!(
        golden == file.bytes,
        "{} differs from its golden file",
        file.name
    );
}

#[test]
fn the_mascot_exports_match_their_golden_files() {
    let files = every_export(&animation(MASCOT)).unwrap();
    let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
    let expected = ["mascot.gif", "mascot.apng", "mascot.png", "mascot.json"];
    assert_eq!(names, [&expected[..], &["mascot-frames.zip"]].concat());
    files.iter().for_each(check_against_golden);
}

#[test]
fn the_blink_exports_match_their_golden_files() {
    let files = every_export(&animation(BLINK)).unwrap();
    let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
    let expected = ["blink-2.gif", "blink-2.apng", "blink-2.png", "blink-2.json"];
    assert_eq!(names, [&expected[..], &["blink-2-frames.zip"]].concat());
    files.iter().for_each(check_against_golden);
}

#[test]
fn exports_carry_their_media_types() {
    let files = every_export(&animation(MASCOT)).unwrap();
    let media_types: Vec<&str> = files.iter().map(|file| file.media_type).collect();
    let expected = [
        "image/gif",
        "image/apng",
        "image/png",
        "application/json",
        "application/zip",
    ];
    assert_eq!(media_types, expected);
}
