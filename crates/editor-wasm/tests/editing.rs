//! The commands of the engine — create, open, restore, apply, undo, redo, save — and the state
//! they publish.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

mod common;

use common::{
    FRAME, LAYER, SIDE, TITLE, add_layer, dot, engine, layer_count, new_animation, operation,
    render, request,
};
use life_pixel_core::{DEFAULT_PALETTE, FrameId, LayerId, Limits};
use life_pixel_editor_wasm::EngineCore;
use life_pixel_editor_wasm::state::{FrameSummary, LayerSummary, Status};
use serde_json::json;

#[test]
fn a_new_engine_is_empty_and_knows_the_limits() {
    let state = EngineCore::default().state();

    assert_eq!(state.status, Status::Empty);
    assert_eq!(state.document, None);
    assert!(!state.can_undo && !state.can_redo && !state.has_unsaved_work);
    assert_eq!(state.limits, Limits::current());
    assert_eq!(EngineCore::limits(), Limits::current());
}

#[test]
fn create_opens_a_saved_animation_of_one_layer_and_one_frame() {
    let state = engine().state();

    assert_eq!(state.status, Status::Ready);
    assert!(!state.can_undo && !state.can_redo && !state.has_unsaved_work);
    let document = state.document.unwrap();
    assert_eq!(document.title, TITLE);
    assert_eq!(
        (u32::from(document.width), u32::from(document.height)),
        (SIDE, SIDE)
    );
    assert_eq!(document.palette, DEFAULT_PALETTE);
    let layer = LayerSummary {
        id: LayerId::new(LAYER),
        name: "Base".to_owned(),
        visible: true,
    };
    assert_eq!(document.layers, [layer]);
    let frame = FrameSummary {
        id: FrameId::new(FRAME),
        duration_ms: 100,
    };
    assert_eq!(document.frames, [frame]);
    assert!(document.tags.is_empty());
}

#[test]
fn create_refuses_options_out_of_the_limits_with_their_codes() {
    let cases = [
        (json!({ "width": 0 }), "document.canvas_size"),
        (json!({ "width": 70_000 }), "document.canvas_size"),
        (json!({ "title": "" }), "document.name"),
        (json!({ "frameDurationMs": 5 }), "document.frame_duration"),
        (
            json!({ "frameDurationMs": 70_000 }),
            "document.frame_duration",
        ),
        (json!({ "palette": ["#ff0000ff"] }), "document.palette"),
        (json!({ "palette": ["red"] }), "document.palette"),
    ];
    for (change, code) in cases {
        let mut options = json!({ "title": TITLE, "width": 4, "height": 4, "layerName": "Base" });
        options
            .as_object_mut()
            .unwrap()
            .extend(change.as_object().unwrap().clone());
        let mut engine = EngineCore::default();

        let error = engine.create(request(options)).unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(engine.state().status, Status::Empty);
    }
}

#[test]
fn create_takes_the_palette_and_duration_it_is_given() {
    let mut engine = EngineCore::default();
    let options = json!({
        "title": TITLE, "width": 2, "height": 3, "layerName": "Base",
        "frameDurationMs": 250, "palette": ["#00000000", "#ff7f27c0"],
    });
    engine.create(request(options)).unwrap();

    let document = engine.state().document.unwrap();
    let palette: Vec<String> = document.palette.iter().map(ToString::to_string).collect();
    assert_eq!(palette, ["#00000000", "#ff7f27c0"]);
    assert_eq!(document.frames[0].duration_ms, 250);
}

#[test]
fn an_operation_changes_the_summary_and_undo_and_redo_toggle_it() {
    let mut engine = engine();
    engine.apply(add_layer()).unwrap();
    let state = engine.state();
    assert_eq!(layer_count(&state), 2);
    assert!(state.can_undo && !state.can_redo);

    assert!(engine.undo());
    let state = engine.state();
    assert_eq!(layer_count(&state), 1);
    assert!(!state.can_undo && state.can_redo);

    assert!(engine.redo());
    let state = engine.state();
    assert_eq!(layer_count(&state), 2);
    assert!(state.can_undo && !state.can_redo);
}

#[test]
fn undo_and_redo_without_a_step_say_so_and_change_nothing() {
    let mut engine = engine();
    engine.apply(add_layer()).unwrap();
    let state = engine.state();

    assert!(!engine.redo());
    assert_eq!(engine.state(), state);
    assert!(engine.undo());
    let undone = engine.state();
    assert!(!engine.undo());
    assert_eq!(engine.state(), undone);
}

/// Operations the test animation refuses, each with its code.
fn refused_operations() -> [(serde_json::Value, &'static str); 5] {
    [
        (
            json!({ "kind": "deleteLayer", "layer": 9_999 }),
            "edit.layer_not_found",
        ),
        (
            json!({ "kind": "deleteLayer", "layer": LAYER }),
            "edit.last_layer",
        ),
        (
            json!({ "kind": "addLayer", "position": 5, "name": "Top" }),
            "edit.position_out_of_range",
        ),
        (
            json!({ "kind": "renameLayer", "layer": LAYER, "name": "" }),
            "document.name",
        ),
        (
            json!({ "kind": "setFrameDuration", "frame": FRAME, "durationMs": 1 }),
            "document.frame_duration",
        ),
    ]
}

#[test]
fn a_refused_operation_changes_nothing_and_gives_its_code() {
    let mut engine = engine();
    engine.apply(dot(1, 1, 1)).unwrap();
    let before = engine.state();
    let pixels = engine.render(&render(None)).unwrap();

    for (change, code) in refused_operations() {
        assert_eq!(engine.apply(operation(change)).unwrap_err().code, code);
    }
    assert_eq!(engine.state(), before);
    assert_eq!(engine.render(&render(None)).unwrap(), pixels);
}

#[test]
fn without_an_animation_every_request_but_the_history_is_refused() {
    let mut engine = EngineCore::default();
    assert!(!engine.undo() && !engine.redo());
    engine.mark_saved();
    assert_eq!(engine.state().status, Status::Empty);

    let no_document = "engine.no_document";
    assert_eq!(engine.apply(add_layer()).unwrap_err().code, no_document);
    assert_eq!(engine.render(&render(None)).unwrap_err().code, no_document);
    assert_eq!(engine.serialize().unwrap_err().code, no_document);
    assert_eq!(
        engine.export(&common::export("gif")).unwrap_err().code,
        no_document
    );
    let snippet = json!({ "framework": "html", "src": "a.wasm", "loader": "l.js", "alt": "" });
    assert_eq!(
        engine.snippet(&request(snippet)).unwrap_err().code,
        no_document
    );
}

#[test]
fn unsaved_work_follows_changes_saves_and_undo() {
    let mut engine = engine();
    engine.apply(dot(0, 0, 1)).unwrap();
    assert!(engine.state().has_unsaved_work);

    engine.mark_saved();
    assert!(!engine.state().has_unsaved_work);

    assert!(engine.undo());
    assert!(engine.state().has_unsaved_work);

    assert!(engine.redo());
    assert!(!engine.state().has_unsaved_work);
}

#[test]
fn open_reads_back_what_serialize_wrote_pixels_included() {
    let mut first = engine();
    first.apply(add_layer()).unwrap();
    first.apply(dot(2, 1, 3)).unwrap();
    let document = first.serialize().unwrap();

    let mut second = EngineCore::default();
    second.open(&document).unwrap();

    let state = second.state();
    assert_eq!(state.document, first.state().document);
    assert!(!state.can_undo && !state.has_unsaved_work);
    assert_eq!(second.render(&render(None)), first.render(&render(None)));
}

#[test]
fn open_refuses_what_is_not_a_document_and_keeps_the_animation() {
    let mut engine = engine();
    let before = engine.state();

    assert_eq!(
        engine.open(b"not json").unwrap_err().code,
        "document.malformed"
    );
    let future = br#"{ "format": "life-pixel/animation", "version": 99 }"#;
    let error = engine.open(future).unwrap_err();
    assert_eq!(error.code, "document.unsupported_version");
    assert_eq!(error.params.get("version"), Some(&json!(99)));
    assert_eq!(engine.state(), before);
}

#[test]
fn a_restored_snapshot_stays_unsaved_until_the_next_save() {
    let mut first = engine();
    first.apply(dot(0, 0, 1)).unwrap();
    let snapshot = first.serialize().unwrap();

    let mut engine = EngineCore::default();
    engine.restore(&snapshot).unwrap();
    let state = engine.state();
    assert!(state.has_unsaved_work && !state.can_undo);
    assert_eq!(state.document, first.state().document);

    engine.apply(add_layer()).unwrap();
    assert!(engine.undo());
    assert!(engine.state().has_unsaved_work);

    engine.mark_saved();
    assert!(!engine.state().has_unsaved_work);
    engine.create(new_animation()).unwrap();
    assert!(!engine.state().has_unsaved_work);
}
