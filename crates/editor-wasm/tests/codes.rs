//! Every code the engine returns of its own — those it creates and those it reuses from the
//! server — has its `errors.<code>` message in every catalogue.

use std::collections::BTreeSet;

use life_pixel_editor_wasm::EngineError;
use life_pixel_editor_wasm::errors::{CODES, REUSED_CODES};
use serde_json::{Map, Value};

const CATALOGUES: [&str; 2] = ["en", "fr"];

fn catalogue(language: &str) -> Map<String, Value> {
    let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn the_engines_own_errors_use_the_codes_it_lists() {
    let errors = [
        EngineError::no_document(),
        EngineError::malformed_request(),
        EngineError::internal(),
    ];
    let listed: BTreeSet<&str> = CODES.iter().chain(REUSED_CODES).copied().collect();
    let used: BTreeSet<&str> = errors.iter().map(|error| error.code).collect();
    assert_eq!(used, listed);
    assert!(errors.iter().all(|error| error.params.is_empty()));
}

#[test]
fn every_code_of_the_engine_has_its_message_in_every_catalogue() {
    for language in CATALOGUES {
        let messages = catalogue(language);
        for code in CODES.iter().chain(REUSED_CODES) {
            let key = format!("errors.{code}");
            assert!(messages.contains_key(&key), "{language}: {key} is missing");
        }
    }
}
