//! The serialized shape of an operation: a `kind` and camelCase fields, the shape the engine
//! interface mirrors.

use life_pixel_core::edit::Operation;
use serde_json::{Value, json};

/// One operation of each kind, as the engine interface writes it.
fn every_kind() -> Vec<Value> {
    let tag = json!({ "name": "idle", "first": 0, "last": 1, "loop": "once" });
    let area = json!({ "x": -1, "y": 2, "width": 3, "height": 4 });
    vec![
        json!({ "kind": "paintStroke", "layer": 1, "frame": 2, "points": [{ "x": 0, "y": -1 }], "index": 3 }),
        json!({ "kind": "fill", "layer": 1, "frame": 2, "at": { "x": 0, "y": 1 }, "index": 3 }),
        json!({ "kind": "line", "layer": 1, "frame": 2, "from": { "x": 0, "y": 1 }, "to": { "x": 2, "y": 3 }, "index": 3 }),
        json!({ "kind": "rectangle", "layer": 1, "frame": 2, "from": { "x": 0, "y": 1 }, "to": { "x": 2, "y": 3 }, "index": 3, "filled": true }),
        json!({ "kind": "moveSelection", "layer": 1, "frame": 2, "area": area, "offset": { "x": 1, "y": -1 } }),
        json!({ "kind": "setPaletteEntry", "index": 1, "color": "#ff7f27c0" }),
        json!({ "kind": "addPaletteEntry", "color": "#00000000" }),
        json!({ "kind": "removePaletteEntry", "index": 1 }),
        json!({ "kind": "movePaletteEntry", "from": 1, "to": 2 }),
        json!({ "kind": "addLayer", "position": 0, "name": "Shadow" }),
        json!({ "kind": "deleteLayer", "layer": 1 }),
        json!({ "kind": "moveLayer", "layer": 1, "position": 0 }),
        json!({ "kind": "renameLayer", "layer": 1, "name": "Body" }),
        json!({ "kind": "setLayerVisibility", "layer": 1, "visible": false }),
        json!({ "kind": "addFrame", "position": 1, "durationMs": 100 }),
        json!({ "kind": "duplicateFrame", "frame": 2 }),
        json!({ "kind": "deleteFrame", "frame": 2 }),
        json!({ "kind": "moveFrame", "frame": 2, "position": 0 }),
        json!({ "kind": "setFrameDuration", "frame": 2, "durationMs": 80 }),
        json!({ "kind": "addTag", "tag": tag }),
        json!({ "kind": "updateTag", "name": "idle", "tag": tag }),
        json!({ "kind": "deleteTag", "name": "idle" }),
        json!({ "kind": "replaceTags", "tags": [tag] }),
        json!({ "kind": "setTitle", "title": "Mascot" }),
        json!({ "kind": "importImage", "layer": 1, "frame": 2, "png": [137, 80, 78, 71], "at": { "x": 0, "y": 0 } }),
        json!({ "kind": "importSpriteSheet", "layer": 1, "position": 1, "png": [137, 80], "cellWidth": 16, "cellHeight": 8, "durationMs": 100 }),
    ]
}

#[test]
fn every_kind_reads_and_writes_the_same_json() {
    let kinds = every_kind();
    assert_eq!(kinds.len(), 26, "one case per operation");
    for case in kinds {
        let operation: Operation =
            serde_json::from_value(case.clone()).unwrap_or_else(|error| panic!("{case}: {error}"));
        assert_eq!(serde_json::to_value(&operation).unwrap(), case);
    }
}

#[test]
fn an_unknown_kind_or_field_a_missing_field_or_a_bad_colour_is_refused() {
    let refused = [
        json!({ "kind": "spray", "layer": 1 }),
        json!({ "kind": "deleteLayer", "layer": 1, "frame": 2 }),
        json!({ "kind": "deleteLayer" }),
        json!({ "kind": "addPaletteEntry", "color": "#ff7f27" }),
        json!({ "kind": "addTag", "tag": { "name": "idle", "first": 0, "last": 0, "loop": "bounce" } }),
        json!({ "kind": "importImage", "layer": 1, "frame": 2, "png": [256], "at": { "x": 0, "y": 0 } }),
    ];
    for case in refused {
        assert!(
            serde_json::from_value::<Operation>(case.clone()).is_err(),
            "{case}"
        );
    }
}
