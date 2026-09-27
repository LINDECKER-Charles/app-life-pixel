//! Animations drawn from text grids, and the checks every operation test shares: undo restores
//! the document byte for byte, and a refused operation leaves it unchanged.

#![allow(dead_code)] // Each test module uses its own share of the helpers.

pub mod golden;
pub mod png;

use life_pixel_core::edit::{self, Operation};
use life_pixel_core::serialize::{grid, read_document, write_document};
use life_pixel_core::{Animation, FrameId, LayerId, Point};
use serde_json::{Value, json};

/// The layer of an animation sketched with one layer.
pub const LAYER: LayerId = LayerId::new(1);
/// The frame of an animation sketched with one layer.
pub const FRAME: FrameId = FrameId::new(2);

/// The default palette, as a document writes it.
const PALETTE: [&str; 16] = [
    "#00000000",
    "#000000ff",
    "#ffffffff",
    "#7f7f7fff",
    "#c3c3c3ff",
    "#880015ff",
    "#ed1c24ff",
    "#ff7f27ff",
    "#fff200ff",
    "#22b14cff",
    "#00a2e8ff",
    "#3f48ccff",
    "#a349a4ff",
    "#b97a57ff",
    "#ffaec9ff",
    "#99d9eaff",
];

/// An animation with the default palette, one frame, and one layer per grid, bottom first:
/// layers take ids 1 to n, the frame n + 1.
pub fn sketch(layers: &[&[&str]]) -> Animation {
    read_document(sketch_document(layers).to_string().as_bytes()).unwrap()
}

/// The document of [`sketch`], to change before reading it.
pub fn sketch_document(layers: &[&[&str]]) -> Value {
    let frame = layers.len() + 1;
    let width = layers[0][0].len();
    let cels: Vec<Value> = (1..)
        .zip(layers)
        .map(|(layer, rows)| json!({ "layer": layer, "frame": frame, "grid": rows }))
        .collect();
    let layer_list: Vec<Value> = (1..=layers.len())
        .map(|id| json!({ "id": id, "name": format!("Layer {id}"), "visible": true }))
        .collect();
    json!({
        "format": "life-pixel/animation",
        "version": 1,
        "title": "Sketch",
        "width": width,
        "height": layers[0].len(),
        "palette": PALETTE,
        "layers": layer_list,
        "frames": [{ "id": frame, "durationMs": 100 }],
        "cels": cels,
        "tags": [],
        "nextId": frame + 1
    })
}

/// A blank animation of one layer and one frame.
pub fn blank(width: usize, height: usize) -> Animation {
    let row = ".".repeat(width);
    let rows = vec![row.as_str(); height];
    sketch(&[&rows])
}

/// The cel of `layer` on `frame` as grid rows; a blank cel is all dots.
pub fn rows(animation: &Animation, layer: LayerId, frame: FrameId) -> Vec<String> {
    let shape = animation.cel_shape();
    let blank = vec![0; shape.pixel_count()];
    let indices = animation
        .cel(layer, frame)
        .map_or(blank.as_slice(), |cel| cel.indices());
    grid::format(indices, shape)
}

/// The document of `animation`, as the writer writes it.
pub fn document(animation: &Animation) -> String {
    write_document(animation).unwrap()
}

/// Applies `operation`; checks that its inverse restores the document byte for byte, and that
/// the inverse of that applies the operation again.
pub fn apply(animation: &mut Animation, operation: &Operation) {
    let before = document(animation);
    let inverse = edit::apply(animation, operation).unwrap();
    let after = document(animation);
    let redo = inverse.apply(animation);
    assert_eq!(document(animation), before, "undo of {operation:?}");
    let _undo = redo.apply(animation);
    assert_eq!(document(animation), after, "redo of {operation:?}");
}

/// Checks that `operation` fails with `code` and leaves the document as it was.
pub fn assert_refused(animation: &mut Animation, operation: &Operation, code: &str) {
    let before = document(animation);
    let error = edit::apply(animation, operation).unwrap_err();
    assert_eq!(error.code(), code, "{operation:?}");
    assert_eq!(document(animation), before, "{operation:?}");
}

/// The point `x`, `y`.
pub fn point(x: i32, y: i32) -> Point {
    Point { x, y }
}
