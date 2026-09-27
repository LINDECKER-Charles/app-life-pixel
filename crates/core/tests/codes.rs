//! Every code of `CODES` is unique, has its `errors.<code>` message in every catalogue, and each
//! message names exactly the params of its code.

use std::collections::BTreeSet;

use life_pixel_core::DocumentError;
use life_pixel_core::edit::EditError;
use life_pixel_core::error::CODES;
use serde_json::{Map, Value};

const CATALOGUES: [&str; 2] = ["en", "fr"];

/// The code and params of one error of each variant, of documents and of edits.
fn every_error() -> Vec<(&'static str, Map<String, Value>)> {
    let documents = document_errors().into_iter().map(EditError::Document);
    let edits = documents.chain(edit_errors());
    edits.map(|error| (error.code(), error.params())).collect()
}

fn edit_errors() -> Vec<EditError> {
    vec![
        EditError::LayerNotFound,
        EditError::FrameNotFound,
        EditError::TagNotFound {
            name: "idle".to_owned(),
        },
        EditError::OutOfCanvas,
        EditError::LastLayer,
        EditError::LastFrame,
        EditError::PaletteFull,
        EditError::PositionOutOfRange { max: 3 },
        EditError::StrokeTooLong,
        EditError::ImageTooLarge,
        EditError::ImageMalformed,
        EditError::SheetGrid,
    ]
}

fn document_errors() -> Vec<DocumentError> {
    vec![
        DocumentError::Malformed,
        DocumentError::UnsupportedVersion { version: 2 },
        DocumentError::TooLarge,
        DocumentError::CanvasSize,
        DocumentError::FrameCount,
        DocumentError::LayerCount,
        DocumentError::TagCount,
        DocumentError::PixelBudget,
        DocumentError::Palette,
        DocumentError::FrameDuration,
        DocumentError::Name,
        DocumentError::Tag {
            name: "idle".to_owned(),
        },
        DocumentError::Reference,
        DocumentError::Cel,
        DocumentError::GridSize {
            width: 2,
            height: 2,
        },
        DocumentError::GridCharacter { row: 0, column: 1 },
        DocumentError::GridIndex {
            row: 0,
            column: 1,
            index: 3,
        },
    ]
}

fn catalogue(language: &str) -> Map<String, Value> {
    let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The names of the ICU arguments of `message`: what follows each `{`.
fn arguments(message: &str) -> BTreeSet<String> {
    message
        .split('{')
        .skip(1)
        .map(|rest| {
            rest.chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect()
        })
        .collect()
}

#[test]
fn codes_are_unique_and_cover_every_error() {
    let unique: BTreeSet<&str> = CODES.iter().copied().collect();
    assert_eq!(unique.len(), CODES.len());
    let used: BTreeSet<&str> = every_error().iter().map(|(code, _)| *code).collect();
    assert_eq!(used, unique);
}

#[test]
fn every_code_has_its_message_with_its_params_in_every_catalogue() {
    for language in CATALOGUES {
        let messages = catalogue(language);
        for (code, params) in every_error() {
            let key = format!("errors.{code}");
            let message = messages.get(&key).and_then(Value::as_str);
            let message = message.unwrap_or_else(|| panic!("{language}: {key} is missing"));
            let params: BTreeSet<String> = params.keys().cloned().collect();
            assert_eq!(arguments(message), params, "{language}: {key}");
        }
    }
}
