//! Every code of `errors::CODES` is unique, is the code of a `LocalWriteError`, and has its
//! `errors.<code>` message naming exactly its params in every catalogue.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

use std::collections::BTreeSet;
use std::path::PathBuf;

use life_pixel_cli::errors::{CODES, LocalWriteError};
use serde_json::{Map, Value};

const CATALOGUES: [&str; 2] = ["en", "fr"];

fn every_error() -> [LocalWriteError; 4] {
    let path = PathBuf::from("out/mascot.gif");
    [
        LocalWriteError::DirectoryNotAllowed(path.clone()),
        LocalWriteError::FileExists(path.clone()),
        LocalWriteError::Symlink(path.clone()),
        LocalWriteError::WriteFailed(path),
    ]
}

fn catalogue(language: &str) -> Map<String, Value> {
    let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The names of the `{argument}`s of `message`.
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
    assert_eq!(unique.len(), CODES.len(), "a code is listed twice");
    let used: BTreeSet<&str> = every_error().iter().map(LocalWriteError::code).collect();
    assert_eq!(used, unique);
}

#[test]
fn every_code_has_its_message_with_its_path_parameter_in_every_catalogue() {
    for language in CATALOGUES {
        let messages = catalogue(language);
        for error in every_error() {
            let key = format!("errors.{}", error.code());
            let message = messages.get(&key).and_then(Value::as_str);
            let message = message.unwrap_or_else(|| panic!("{language}: {key} is missing"));
            let params: BTreeSet<String> = error.params().keys().cloned().collect();
            assert_eq!(arguments(message), params, "{language}: {key}");
        }
    }
}
