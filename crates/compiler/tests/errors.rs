//! Each refusal of an export, with its code and params, and each code's message in every
//! catalogue.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use std::collections::BTreeSet;

use common::{MASCOT, animation, options};
use life_pixel_compiler::error::CODES;
use life_pixel_compiler::{
    ClassicOptions, ExportError, export_apng, export_gif, export_png_frames, export_sprite_sheet,
};
use life_pixel_core::Animation;
use life_pixel_core::serialize::read_document;
use serde_json::{Map, Value, json};

const CATALOGUES: [&str; 2] = ["en", "fr"];

/// The error of every export with `options`, which must all fail alike.
fn refusal(animation: &Animation, options: &ClassicOptions) -> ExportError {
    let errors = [
        export_gif(animation, options).unwrap_err(),
        export_apng(animation, options).unwrap_err(),
        export_sprite_sheet(animation, options).unwrap_err(),
        export_png_frames(animation, options).unwrap_err(),
    ];
    assert!(errors.iter().all(|error| *error == errors[0]), "{errors:?}");
    errors[0].clone()
}

/// A `side × side` animation of `frame_count` blank frames.
fn blank(side: u16, frame_count: usize) -> Animation {
    let frames: Vec<Value> = (0..frame_count)
        .map(|index| json!({ "id": index + 2, "durationMs": 100 }))
        .collect();
    let document = json!({
        "format": "life-pixel/animation", "version": 1, "title": "Big",
        "width": side, "height": side, "palette": ["#00000000"],
        "layers": [{ "id": 1, "name": "Layer", "visible": true }],
        "frames": frames, "cels": [], "tags": [], "nextId": frame_count + 2
    });
    read_document(document.to_string().as_bytes()).unwrap()
}

fn params(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn a_scale_out_of_1_to_16_is_refused() {
    let mascot = animation(MASCOT);
    for scale in [0, 17] {
        let error = refusal(&mascot, &options(None, scale));
        assert_eq!(error, ExportError::Scale);
        assert_eq!(error.code(), "export.scale");
        assert_eq!(error.params(), params(json!({ "min": 1, "max": 16 })));
    }
}

#[test]
fn an_unknown_tag_is_refused() {
    let error = refusal(&animation(MASCOT), &options(Some("run"), 1));
    assert_eq!(error.code(), "export.tag_not_found");
    assert_eq!(error.params(), params(json!({ "name": "run" })));
}

#[test]
fn a_sprite_sheet_wider_than_the_largest_side_is_refused() {
    let big = blank(512, 2);
    let error = export_sprite_sheet(&big, &options(None, 16)).unwrap_err();
    assert_eq!(error.code(), "export.too_large");
    assert_eq!(error.params(), params(json!({ "maxSide": 8_192 })));
    let one_frame = blank(512, 1);
    let sheet = export_sprite_sheet(&one_frame, &options(None, 16));
    assert!(sheet.is_ok(), "8,192 pixels is the largest side allowed");
}

#[test]
fn codes_are_unique_and_each_has_its_message_with_its_params_in_every_catalogue() {
    let errors = [
        ExportError::Scale,
        ExportError::TooLarge,
        ExportError::TagNotFound {
            name: "run".to_owned(),
        },
    ];
    let unique: BTreeSet<&str> = CODES.iter().copied().collect();
    let used: BTreeSet<&str> = errors.iter().map(ExportError::code).collect();
    assert_eq!((unique.len(), &used), (CODES.len(), &unique));
    for language in CATALOGUES {
        let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(&path).unwrap();
        let messages: Map<String, Value> = serde_json::from_str(&text).unwrap();
        for error in &errors {
            let key = format!("errors.{}", error.code());
            let message = messages.get(&key).and_then(Value::as_str);
            let message = message.unwrap_or_else(|| panic!("{language}: {key} is missing"));
            let expected: BTreeSet<String> = error.params().keys().cloned().collect();
            assert_eq!(arguments(message), expected, "{language}: {key}");
        }
    }
}

/// The names of the ICU arguments of `message`: what follows each `{`.
fn arguments(message: &str) -> BTreeSet<String> {
    let names = message.split('{').skip(1);
    names
        .map(|rest| {
            rest.chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect()
        })
        .collect()
}
