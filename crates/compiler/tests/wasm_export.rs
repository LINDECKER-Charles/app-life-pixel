//! The WebAssembly export of each fixture animation, byte for byte against `tests/golden/`,
//! beside the frames it must show — `<stem>.expected.json`, the RGBA of `life-pixel-core`'s
//! rendering, which `player-js/tests/golden-exports.spec.ts` compares in a browser. Every golden
//! export embeds the player: after a deliberate change of the player or of the encoder, rewrite
//! them with `LIFE_PIXEL_UPDATE_GOLDEN=1 cargo test -p life-pixel-compiler --test wasm_export`,
//! then check that `wasm_playback` still passes, since it says what the bytes must show.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::path::PathBuf;

use common::{MASCOT, SCANNER, WASM_FIXTURES, animation};
use life_pixel_compiler::{PLAYER_WASM, export_wasm, file_stem};
use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{Animation, render};
use life_pixel_format::{Payload, Rgba};
use serde_json::json;
use sha2::{Digest, Sha256};

const UPDATE_VARIABLE: &str = "LIFE_PIXEL_UPDATE_GOLDEN";
const SECTION_NAME: &[u8] = b"life-pixel";
const CUSTOM_SECTION_ID: u8 = 0x00;

fn golden_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "golden", name]
        .iter()
        .collect()
}

fn check_against_golden(name: &str, bytes: &[u8]) {
    let path = golden_path(name);
    if std::env::var_os(UPDATE_VARIABLE).is_some() {
        std::fs::write(&path, bytes).unwrap();
    }
    let golden = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    assert!(
        golden == bytes,
        "{name} differs from its golden file: after a deliberate change of the player or the \
         encoder, rerun with {UPDATE_VARIABLE}=1"
    );
}

/// What the export of `animation` must show, as `golden-exports.spec.ts` reads it: the title, the
/// size, the tags, and each frame's duration and RGBA in hexadecimal.
fn expected_frames(animation: &Animation) -> String {
    let tags = animation.tags().iter().map(|tag| {
        json!({ "name": tag.name(), "first": tag.first(), "last": tag.last(),
                "loop": tag.loop_mode() })
    });
    let frames = animation.frames().iter().map(|frame| {
        let rgba = render::rgba(animation, frame.id());
        let hex: String = rgba.iter().map(|byte| format!("{byte:02x}")).collect();
        json!({ "durationMs": frame.duration_ms(), "rgba": hex })
    });
    let expected = json!({
        "title": animation.title().as_str(),
        "width": animation.width(),
        "height": animation.height(),
        "tags": tags.collect::<Vec<_>>(),
        "frames": frames.collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&expected).unwrap() + "\n"
}

/// Reads an unsigned LEB128 from the front of `bytes`.
fn read_leb128(bytes: &mut &[u8]) -> usize {
    let mut value = 0;
    for shift in (0..).step_by(7) {
        let (&byte, rest) = bytes.split_first().unwrap();
        *bytes = rest;
        value |= usize::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            break;
        }
    }
    value
}

#[test]
fn every_wasm_export_matches_its_golden_file() {
    let mut names = Vec::new();
    for document in WASM_FIXTURES {
        let animation = animation(document);
        let stem = file_stem(animation.title().as_str());
        check_against_golden(&format!("{stem}.wasm"), &export_wasm(&animation).unwrap());
        check_against_golden(
            &format!("{stem}.expected.json"),
            expected_frames(&animation).as_bytes(),
        );
        names.push(stem);
    }
    assert_eq!(names, ["mascot", "blink-2", "scanner"]);
}

#[test]
fn an_export_is_the_player_followed_by_the_life_pixel_section() {
    let export = export_wasm(&animation(SCANNER)).unwrap();

    let (player, mut section) = export.split_at(PLAYER_WASM.len());
    assert_eq!(player, PLAYER_WASM);
    let (&id, rest) = section.split_first().unwrap();
    assert_eq!(id, CUSTOM_SECTION_ID);
    assert!(
        rest[0] & 0x80 != 0,
        "the section's size spans several bytes"
    );
    section = rest;
    let size = read_leb128(&mut section);
    assert_eq!(size, section.len(), "the section ends the module");
    let name_length = read_leb128(&mut section);
    let (name, payload) = section.split_at(name_length);
    assert_eq!(name, SECTION_NAME);
    let payload = Payload::parse(payload).unwrap();
    let shape = (payload.width(), payload.height(), payload.frame_count());
    assert_eq!(shape, (16, 8, 5));
    assert_eq!(payload.title(), "Scanner");
}

#[test]
fn the_same_document_always_exports_to_the_same_bytes() {
    for document in WASM_FIXTURES {
        let first = export_wasm(&animation(document)).unwrap();
        let rewritten = write_document(&animation(document)).unwrap();
        let second = export_wasm(&read_document(rewritten.as_bytes()).unwrap()).unwrap();

        assert!(first == second, "two exports of one document differ");
        assert!(first == export_wasm(&animation(document)).unwrap());
    }
}

#[test]
fn the_embedded_player_is_the_one_whose_hash_is_committed() {
    let hash_file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "player", "player.sha256"]
        .iter()
        .collect();
    let hash_file = std::fs::read_to_string(hash_file).unwrap();
    let committed = hash_file.split_whitespace().next().unwrap();

    let digest = Sha256::digest(PLAYER_WASM);
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();

    assert_eq!(
        hash, committed,
        "the player built for the compiler is not the one crates/player/player.sha256 names"
    );
}

#[test]
fn a_transparent_entry_0_of_any_colour_is_written_all_zeros() {
    let document = MASCOT.replacen("#00000000", "#ff000000", 1);
    let animation = read_document(document.as_bytes()).unwrap();
    assert_eq!(animation.palette().entries()[0].to_bytes(), [255, 0, 0, 0]);

    let export = export_wasm(&animation).unwrap();

    let payload = common::wasm_player::payload_of(&export);
    let payload = Payload::parse(&payload).unwrap();
    assert_eq!(payload.palette_entry(0), Some(Rgba::default()));
    let red = Rgba {
        r: 255,
        g: 0,
        b: 0,
        a: 128,
    };
    assert_eq!(payload.palette_entry(3), Some(red));
}
