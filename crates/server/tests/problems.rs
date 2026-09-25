//! Problems: the status of every code from one table, the shape of every error, and the
//! message of each of the server's own codes in every catalogue.

mod common;

use std::collections::BTreeSet;

use axum::response::IntoResponse;
use life_pixel_server::http::problem::{CODES, Problem, codes, status_of};
use life_pixel_service::library::LibraryError;
use serde_json::{Map, Value, json};

const CATALOGUES: [&str; 2] = ["en", "fr"];

/// The status of every code outside `core`: the service's, the compiler's, those later tasks
/// create, and the server's own.
const STATUSES: [(&str, u16); 32] = [
    ("account.language", 422),
    ("auth.account_suspended", 403),
    ("auth.csrf", 403),
    ("auth.current_password", 403),
    ("auth.email_invalid", 422),
    ("auth.email_taken", 409),
    ("auth.invalid_credentials", 401),
    ("auth.password_length", 422),
    ("auth.token_invalid", 400),
    ("auth.unauthenticated", 401),
    ("client.update_required", 426),
    ("document.version_conflict", 412),
    ("document.version_required", 428),
    ("draw.too_many_operations", 422),
    ("edit.palette_in_use", 422),
    ("export.scale", 422),
    ("export.tag_not_found", 422),
    ("export.too_large", 422),
    ("internal.error", 500),
    ("library.animation_not_found", 404),
    ("library.project_not_found", 404),
    ("library.unavailable", 503),
    ("library.unsupported_version", 422),
    ("preview.too_large", 422),
    ("quota.storage_exceeded", 409),
    ("rate_limit.exceeded", 429),
    ("request.malformed", 400),
    ("request.method_not_allowed", 405),
    ("request.not_found", 404),
    ("request.too_large", 413),
    ("request.unsupported_media_type", 415),
    ("service.unavailable", 503),
];

/// The params of the server's own codes.
const PARAMS: [(&str, &[&str]); 9] = [
    ("auth.csrf", &[]),
    ("client.update_required", &["minimum"]),
    ("document.version_required", &[]),
    ("internal.error", &[]),
    ("rate_limit.exceeded", &["retryAfterSeconds"]),
    ("request.method_not_allowed", &[]),
    ("request.not_found", &[]),
    ("request.too_large", &["maxBytes"]),
    ("request.unsupported_media_type", &[]),
];

/// Every code with the status it must have: `core`'s are `422`, but a document too large.
fn expected_statuses() -> Vec<(&'static str, u16)> {
    let core = life_pixel_core::error::CODES.iter().map(|&code| {
        let status = if code == "document.too_large" {
            413
        } else {
            422
        };
        (code, status)
    });
    core.chain(STATUSES).collect()
}

fn catalogue(language: &str) -> Map<String, Value> {
    let path = format!("{}/../../i18n/{language}.json", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The names of the ICU arguments at the top level of `message`.
fn arguments(message: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut depth = 0_usize;
    for (index, character) in message.char_indices() {
        match character {
            '{' if depth == 0 => {
                let name = message[index + 1..]
                    .chars()
                    .take_while(char::is_ascii_alphanumeric);
                names.insert(name.collect());
                depth += 1;
            }
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    names
}

#[test]
fn every_code_has_the_status_of_the_table() {
    let expected = expected_statuses();
    let service = life_pixel_service::error::CODES.iter();
    for code in service.chain(CODES) {
        assert!(
            expected.iter().any(|(known, _)| known == code),
            "{code} is not tested"
        );
    }
    for (code, status) in expected {
        let found = status_of(code).map(|status| status.as_u16());
        assert_eq!(found, Some(status), "{code}");
        assert_eq!(Problem::new(code).status.as_u16(), status, "{code}");
    }
}

#[tokio::test]
async fn every_error_is_a_problem_document_without_a_sentence() {
    for (code, status) in expected_statuses() {
        let response = Problem::new(code).into_response();
        let answer = common::Answer::read(response).await;
        let params = answer.assert_problem(status, code);
        assert_eq!(params, json!({}));
        assert_eq!(answer.json().as_object().unwrap().len(), 4, "{code}");
    }
}

#[tokio::test]
async fn a_coded_error_keeps_its_params_and_a_wait_its_header() {
    let quota = LibraryError::QuotaExceeded {
        used: 99,
        limit: 100,
        requested: 2,
    };
    let answer = common::Answer::read(Problem::from(quota).into_response()).await;
    let params = answer.assert_problem(409, "quota.storage_exceeded");
    assert_eq!(params, json!({ "used": 99, "limit": 100, "requested": 2 }));
    let wait = Problem::new(codes::RATE_LIMIT_EXCEEDED)
        .with_param("retryAfterSeconds", 30)
        .with_retry_after(30);
    let answer = common::Answer::read(wait.into_response()).await;
    assert_eq!(answer.header("retry-after"), "30");
}

#[test]
fn a_code_without_a_status_is_an_internal_error() {
    let problem = Problem::new("nobody.knows");
    assert_eq!(problem.code, codes::INTERNAL_ERROR);
    assert_eq!(problem.status.as_u16(), 500);
}

#[test]
fn every_server_code_has_its_message_with_its_params_in_every_catalogue() {
    let listed: BTreeSet<&str> = PARAMS.iter().map(|(code, _)| *code).collect();
    assert_eq!(listed, CODES.iter().copied().collect());
    for language in CATALOGUES {
        let messages = catalogue(language);
        for (code, params) in PARAMS {
            let key = format!("errors.{code}");
            let message = messages.get(&key).and_then(Value::as_str);
            let message = message.unwrap_or_else(|| panic!("{language}: {key} is missing"));
            let params: BTreeSet<String> = params.iter().map(|&name| name.to_owned()).collect();
            assert_eq!(arguments(message), params, "{language}: {key}");
        }
    }
}
