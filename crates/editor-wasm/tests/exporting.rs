//! The exports and the snippet, from `compiler`: the WebAssembly bundle with its loader, the files
//! of every classic format, and the refusals.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::{dot, engine, export, request};
use life_pixel_compiler::{
    Framework, LOADER_JS, PLAYER_WASM, SnippetInput, file_stem, render_snippet,
};
use life_pixel_editor_wasm::export::ExportResult;
use serde_json::json;

/// The names and media types of an export's files.
fn names(result: &ExportResult) -> Vec<(&str, &str)> {
    let files = result.files.iter();
    files
        .map(|file| (file.name.as_str(), file.media_type))
        .collect()
}

#[test]
fn the_wasm_export_yields_the_bundle_named_after_the_title_and_the_loader() {
    let mut engine = engine();
    engine.apply(dot(0, 0, 1)).unwrap();

    let result = engine.export(&export("wasm")).unwrap();
    assert_eq!(
        names(&result),
        [
            ("engine-test.wasm", "application/wasm"),
            ("life-pixel.js", "text/javascript")
        ]
    );
    assert!(result.files[0].bytes.starts_with(PLAYER_WASM));
    assert!(result.files[0].bytes.len() > PLAYER_WASM.len());
    assert_eq!(result.files[1].bytes, LOADER_JS.as_bytes());
    let total: usize = result.files.iter().map(|file| file.bytes.len()).sum();
    assert_eq!(result.total_bytes, total);
}

#[test]
fn every_classic_format_answers_with_its_files() {
    let engine = engine();
    let expected = [
        ("gif", vec![("engine-test.gif", "image/gif")]),
        ("apng", vec![("engine-test.apng", "image/apng")]),
        (
            "sprite_sheet",
            vec![
                ("engine-test.png", "image/png"),
                ("engine-test.json", "application/json"),
            ],
        ),
        (
            "png_frames",
            vec![("engine-test-frames.zip", "application/zip")],
        ),
    ];
    for (format, files) in expected {
        let result = engine.export(&export(format)).unwrap();
        assert_eq!(names(&result), files, "{format}");
        assert!(
            result.files.iter().all(|file| !file.bytes.is_empty()),
            "{format}"
        );
    }
}

#[test]
fn a_classic_export_takes_the_scale_and_the_tag_it_is_given() {
    let mut engine = engine();
    let add_frame = json!({ "kind": "addFrame", "position": 1, "durationMs": 100 });
    engine.apply(request(add_frame)).unwrap();
    let tag = json!({ "name": "idle", "first": 1, "last": 1, "loop": "once" });
    engine
        .apply(request(json!({ "kind": "addTag", "tag": tag })))
        .unwrap();

    let sheet = json!({ "format": "sprite_sheet", "scale": 3, "tag": "idle" });
    let result = engine.export(&request(sheet)).unwrap();
    let description: serde_json::Value = serde_json::from_slice(&result.files[1].bytes).unwrap();
    let frames = description["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["sourceSize"], json!({ "w": 12, "h": 12 }));
}

#[test]
fn an_export_the_compiler_refuses_gives_its_code() {
    let engine = engine();
    let cases = [
        (json!({ "format": "gif", "scale": 0 }), "export.scale"),
        (json!({ "format": "apng", "scale": 17 }), "export.scale"),
        (
            json!({ "format": "png_frames", "scale": 300 }),
            "export.scale",
        ),
        (
            json!({ "format": "sprite_sheet", "tag": "missing" }),
            "export.tag_not_found",
        ),
    ];
    for (export_request, code) in cases {
        let error = engine.export(&request(export_request)).unwrap_err();
        assert_eq!(error.code, code);
    }
}

#[test]
fn the_snippet_is_the_compilers_named_after_the_title() {
    let engine = engine();
    for framework in ["html", "angular", "react", "vue"] {
        let snippet_request = json!({
            "framework": framework, "src": "/assets/engine-test.wasm",
            "loader": "/assets/life-pixel.js", "tag": "idle", "alt": "A test",
        });
        let snippet = engine.snippet(&request(snippet_request)).unwrap();

        let framework: Framework = request(json!(framework));
        let input = SnippetInput {
            src: "/assets/engine-test.wasm".to_owned(),
            loader: "/assets/life-pixel.js".to_owned(),
            tag: Some("idle".to_owned()),
            alt: "A test".to_owned(),
            file_stem: file_stem(common::TITLE),
        };
        assert_eq!(snippet, render_snippet(framework, &input));
    }
}
