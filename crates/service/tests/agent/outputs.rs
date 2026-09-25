//! `export` in every format, with `export_completed`, and `snippet` for every framework.

use life_pixel_compiler::{
    ExportFormat, Framework, LOADER_JS, PLAYER_WASM, SnippetInput, render_snippet,
};
use life_pixel_core::LoopMode;
use life_pixel_core::edit::TagSpec;
use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MIN_SCALE};
use life_pixel_service::animation::{ExportRequest, SetTagsRequest, SnippetRequest};
use life_pixel_service::ports::ProductEvent;
use life_pixel_service::{Owner, ProjectId};
use serde_json::json;

use super::support::{Fixture, assert_coded};
use crate::common::{ACCOUNT, spec};

fn export(fixture: &Fixture, format: ExportFormat) -> ExportRequest {
    ExportRequest {
        id: fixture.id,
        format,
        tag: None,
        scale: None,
    }
}

fn snippet(fixture: &Fixture, framework: Framework) -> SnippetRequest {
    SnippetRequest {
        id: fixture.id,
        framework,
        src: None,
        loader: None,
        tag: None,
        alt: None,
    }
}

/// A 4 × 4 "Mascot" with a tag "idle" on its only frame.
async fn tagged() -> Fixture {
    let fixture = Fixture::blank(4, 4).await;
    let idle = TagSpec {
        name: "idle".to_owned(),
        first: 0,
        last: 0,
        loop_mode: LoopMode::Loop,
    };
    let request = SetTagsRequest {
        id: fixture.id,
        tags: vec![idle],
    };
    fixture.editing.set_tags(&ACCOUNT, request).await.unwrap();
    fixture
}

#[tokio::test]
async fn every_format_exports_its_files() {
    let fixture = tagged().await;
    fixture
        .write(0, &["1...", ".2..", "..3.", "...4"])
        .await
        .unwrap();
    let formats = [
        (
            ExportFormat::Wasm,
            vec!["mascot.wasm", "life-pixel.js"],
            &b"\0asm"[..],
        ),
        (ExportFormat::Gif, vec!["mascot.gif"], b"GIF89a"),
        (ExportFormat::Apng, vec!["mascot.apng"], b"\x89PNG"),
        (
            ExportFormat::SpriteSheet,
            vec!["mascot.png", "mascot.json"],
            b"\x89PNG",
        ),
        (ExportFormat::PngFrames, vec!["mascot-frames.zip"], b"PK"),
    ];

    for (format, names, magic) in formats {
        let files = fixture.editing.export(&ACCOUNT, export(&fixture, format));

        let files = files.await.unwrap();
        let file_names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(file_names, names, "{format:?}");
        assert!(files[0].bytes.starts_with(magic), "{format:?}");
    }
}

#[tokio::test]
async fn the_wasm_export_is_the_player_with_the_payload_and_its_loader() {
    let fixture = tagged().await;
    let request = ExportRequest {
        tag: Some("idle".to_owned()),
        ..export(&fixture, ExportFormat::Wasm)
    };

    let files = fixture.editing.export(&ACCOUNT, request).await.unwrap();

    assert!(files[0].bytes.starts_with(PLAYER_WASM));
    assert!(files[0].bytes.len() > PLAYER_WASM.len());
    assert_eq!(files[0].media_type, "application/wasm");
    assert_eq!(files[1].bytes, LOADER_JS.as_bytes());
    assert_eq!(files[1].media_type, "text/javascript");
}

#[tokio::test]
async fn an_export_records_its_format_size_and_source() {
    let fixture = tagged().await;

    let files = fixture
        .editing
        .export(&ACCOUNT, export(&fixture, ExportFormat::Gif));

    let bytes = files.await.unwrap()[0].bytes.len() as u64;
    let completed = ProductEvent {
        name: "export_completed",
        account: ACCOUNT.account(),
        properties: vec![
            ("format", "gif".to_owned()),
            ("size", ProductEvent::size_class(bytes).to_owned()),
            ("source", "mcp".to_owned()),
        ],
    };
    assert_eq!(fixture.harness.events().last(), Some(&completed));
}

#[tokio::test]
async fn the_local_library_records_no_export() {
    let fixture = Fixture::blank(2, 2).await;
    let library = &fixture.harness.library;
    let project: ProjectId = library
        .create_project(&Owner::Local, "Mine")
        .await
        .unwrap()
        .id;
    let local = library.create_animation(&Owner::Local, project, spec("Local", 2, 2));
    let local = local.await.unwrap();
    let events_before = fixture.harness.events().len();

    let request = ExportRequest {
        id: local.id,
        ..export(&fixture, ExportFormat::Apng)
    };
    let files = fixture
        .editing
        .export(&Owner::Local, request)
        .await
        .unwrap();

    assert_eq!(files[0].name, "local.apng");
    assert_eq!(fixture.harness.events().len(), events_before);
}

#[tokio::test]
async fn an_export_refuses_a_missing_tag_or_a_scale_out_of_bounds() {
    let fixture = tagged().await;
    let with = |format, tag: &str, scale| ExportRequest {
        tag: Some(tag.to_owned()),
        scale,
        ..export(&fixture, format)
    };
    let events_before = fixture.harness.events().len();

    let wasm = fixture
        .editing
        .export(&ACCOUNT, with(ExportFormat::Wasm, "run", None));
    let gif = fixture
        .editing
        .export(&ACCOUNT, with(ExportFormat::Gif, "run", None));
    let scale = fixture
        .editing
        .export(&ACCOUNT, with(ExportFormat::Gif, "idle", Some(0)));

    let missing = json!({ "name": "run" });
    assert_coded(
        &wasm.await.unwrap_err(),
        "export.tag_not_found",
        missing.clone(),
    );
    assert_coded(&gif.await.unwrap_err(), "export.tag_not_found", missing);
    let bounds = json!({ "min": EXPORT_MIN_SCALE, "max": EXPORT_MAX_SCALE });
    assert_coded(&scale.await.unwrap_err(), "export.scale", bounds);
    assert_eq!(fixture.harness.events().len(), events_before);
}

/// What a snippet of "Mascot" is filled with when the request names nothing.
fn default_input() -> SnippetInput {
    SnippetInput {
        src: "/assets/mascot.wasm".to_owned(),
        loader: "/assets/life-pixel.js".to_owned(),
        tag: None,
        alt: "Mascot".to_owned(),
        file_stem: "mascot".to_owned(),
    }
}

#[tokio::test]
async fn a_snippet_comes_from_the_templates_for_every_framework() {
    let fixture = tagged().await;
    let second_files = [
        (Framework::Html, "life-pixel.js"),
        (Framework::Angular, "mascot-animation.component.ts"),
        (Framework::React, "mascot-animation.tsx"),
        (Framework::Vue, "mascot-animation.vue"),
    ];

    for (framework, second_file) in second_files {
        let snippet = fixture
            .editing
            .snippet(&ACCOUNT, snippet(&fixture, framework));

        let snippet = snippet.await.unwrap();
        let expected = render_snippet(framework, &default_input());
        assert_eq!(snippet.code, expected, "{framework:?}");
        assert!(snippet.code.contains("<life-pixel"), "{framework:?}");
        let names: Vec<&str> = snippet.files.iter().map(|file| &*file.name).collect();
        assert_eq!(names, ["mascot.wasm", second_file], "{framework:?}");
        assert!(snippet.files[0].placement.contains("/assets/mascot.wasm"));
    }
}

#[tokio::test]
async fn a_snippet_takes_the_values_asked_for() {
    let fixture = tagged().await;
    let request = SnippetRequest {
        src: Some("/anim/m.wasm".to_owned()),
        loader: Some("/js/life-pixel.js".to_owned()),
        tag: Some("idle".to_owned()),
        alt: Some("Our mascot".to_owned()),
        ..snippet(&fixture, Framework::Html)
    };

    let snippet = fixture.editing.snippet(&ACCOUNT, request).await.unwrap();

    let expected = "<script type=\"module\" src=\"/js/life-pixel.js\"></script>\n\
                    <life-pixel src=\"/anim/m.wasm\" tag=\"idle\" alt=\"Our mascot\"></life-pixel>";
    assert_eq!(snippet.code.trim_end(), expected);
    assert!(snippet.files[1].placement.contains("/js/life-pixel.js"));
    let json = serde_json::to_value(&snippet).unwrap();
    assert_eq!(json["files"][0]["name"], "mascot.wasm");
}

#[tokio::test]
async fn a_snippet_refuses_a_missing_tag() {
    let fixture = tagged().await;
    let request = SnippetRequest {
        tag: Some("run".to_owned()),
        ..snippet(&fixture, Framework::React)
    };

    let error = fixture
        .editing
        .snippet(&ACCOUNT, request)
        .await
        .unwrap_err();

    assert_coded(&error, "export.tag_not_found", json!({ "name": "run" }));
}
