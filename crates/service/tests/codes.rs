//! Every code of `CODES` is unique, is the code of an error of this crate, and has its
//! `errors.<code>` message naming exactly its params in every catalogue.

use std::collections::BTreeSet;

use life_pixel_service::accounts::AccountsError;
use life_pixel_service::admin::AdminError;
use life_pixel_service::animation::EditingError;
use life_pixel_service::error::CODES;
use life_pixel_service::library::LibraryError;
use life_pixel_service::local::LocalLibraryError;
use life_pixel_service::paging::MalformedCursor;
use life_pixel_service::support::SupportError;
use life_pixel_service::{Coded, CodedError};
use serde_json::{Map, Value};

const CATALOGUES: [&str; 2] = ["en", "fr"];

/// One error of each of the crate's own codes.
fn every_error() -> Vec<CodedError> {
    let library = [
        LibraryError::ProjectNotFound,
        LibraryError::AnimationNotFound,
        LibraryError::VersionConflict { current: 4 },
        LibraryError::QuotaExceeded {
            used: 99,
            limit: 100,
            requested: 2,
        },
        LibraryError::Unavailable,
    ];
    let local = [
        LocalLibraryError::UnsupportedVersion { version: 2 },
        LocalLibraryError::Unavailable {
            path: "Life Pixel".into(),
        },
    ];
    let editing = [
        EditingError::TooManyOperations,
        EditingError::PaletteInUse { index: 3 },
        EditingError::PreviewTooLarge,
    ];
    let library = library.iter().map(CodedError::of);
    let local = local.iter().map(CodedError::of);
    let editing = editing.iter().map(CodedError::of);
    let others = [CodedError::of(&MalformedCursor)];
    let known = library.chain(local).chain(editing).chain(others);
    let hosted = account_errors().into_iter().chain(support_errors());
    known.chain(hosted).chain(admin_errors()).collect()
}

/// One error of each of the internal admin API's codes.
fn admin_errors() -> Vec<CodedError> {
    let errors = [
        AdminError::UserNotFound,
        AdminError::ReasonLength,
        AdminError::ScreenshotNotFound,
    ];
    errors.iter().map(CodedError::of).collect()
}

/// One error of each of the support codes.
fn support_errors() -> Vec<CodedError> {
    let errors = [
        SupportError::RequestNotFound,
        SupportError::Category,
        SupportError::MessageLength,
        SupportError::Screenshot,
        SupportError::RequestClosed,
    ];
    errors.iter().map(CodedError::of).collect()
}

/// One error of each of the accounts' codes.
fn account_errors() -> Vec<CodedError> {
    let errors = [
        AccountsError::Unauthenticated,
        AccountsError::InvalidCredentials,
        AccountsError::EmailInvalid,
        AccountsError::PasswordLength,
        AccountsError::EmailTaken,
        AccountsError::TokenInvalid,
        AccountsError::CurrentPassword,
        AccountsError::AccountSuspended,
        AccountsError::Language {
            available: vec!["en".into(), "fr".into()],
        },
    ];
    errors.iter().map(CodedError::of).collect()
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
    assert_eq!(unique.len(), CODES.len(), "a code is listed twice");
    let used: BTreeSet<&str> = every_error().iter().map(Coded::code).collect();
    assert_eq!(used, unique);
}

#[test]
fn every_code_has_its_message_with_its_params_in_every_catalogue() {
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

#[test]
fn a_document_error_keeps_the_code_of_core() {
    let error = LibraryError::from(life_pixel_core::DocumentError::TooLarge);
    let coded = CodedError::from(error);
    assert_eq!(coded.code, "document.too_large");
    assert!(coded.params.contains_key("maxBytes"));
}
