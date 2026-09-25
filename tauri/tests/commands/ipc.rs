//! Commands called through the IPC, as the app calls them: camelCase arguments, answers and
//! errors as JSON.

use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response};
use tauri::webview::InvokeRequest;
use tauri::{WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::harness::{Harness, document};

fn window(harness: &Harness) -> WebviewWindow<MockRuntime> {
    let builder = WebviewWindowBuilder::new(&harness.app, "main", WebviewUrl::default());
    builder.build().unwrap()
}

/// The answer of the command `command` to `arguments`, or its error.
fn invoke(
    window: &WebviewWindow<MockRuntime>,
    command: &str,
    arguments: Value,
) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: window.url().unwrap(),
        body: InvokeBody::Json(arguments),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_owned(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize().unwrap())
}

#[test]
fn arguments_are_camel_case_and_errors_carry_a_code() {
    let harness = Harness::with_commands();
    let window = window(&harness);
    let project = invoke(&window, "library_create_project", json!({ "name": "Walk" })).unwrap();
    let project_id = project["id"].clone();
    let arguments = json!({ "projectId": project_id, "document": document("Step") });
    let animation = invoke(&window, "library_create_animation", arguments).unwrap();
    assert_eq!(animation["projectId"], project_id);

    let listed = invoke(
        &window,
        "library_list_animations",
        json!({ "projectId": project_id }),
    );
    assert_eq!(listed.unwrap()["items"][0]["title"], "Step");
    let stale = json!({ "id": animation["id"], "version": 0, "document": document("Step") });
    let error = invoke(&window, "library_save_document", stale).unwrap_err();
    assert_eq!(error["code"], "document.version_conflict");
    assert_eq!(error["params"]["current"], animation["version"]);
}

/// Every command of `build.rs`'s manifest, allowed by the capability and registered: it answers,
/// or fails with a code or for a missing argument — never as unknown or not allowed.
#[test]
fn every_command_of_the_manifest_is_allowed_and_registered() {
    let harness = Harness::with_commands();
    let window = window(&harness);
    let build = include_str!("../../build.rs");
    let commands = build
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"')?.strip_suffix("\","));
    let mut count = 0;
    for command in commands {
        count += 1;
        let Err(error) = invoke(&window, command, json!({})) else {
            continue;
        };
        let is_coded = error.get("code").is_some();
        let is_missing_argument = error.as_str().is_some_and(|text| text.contains("missing"));
        assert!(is_coded || is_missing_argument, "{command}: {error}");
    }
    assert_eq!(count, 19);
}
